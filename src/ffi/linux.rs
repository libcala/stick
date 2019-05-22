use std::ffi::CString;
use std::fs;
use std::mem;

extern "C" {
    fn open(pathname: *const u8, flags: i32) -> i32;
    fn close(fd: i32) -> i32;
    fn fcntl(fd: i32, cmd: i32, v: i32) -> i32;
}

#[repr(C)]
struct Device {
    name: [u8; 256 + 17],
    fd: i32,
}

/*impl PartialEq for Device {
    fn eq(&self, other: &Device) -> bool {
        self.name == other.name;
    }
}*/

pub struct NativeManager {
    // Inotify File descriptor.
    pub(crate) inotify: i32,
    // Epoll File Descriptor.
    pub(crate) fd: i32,
    // Controller File Descriptors.
    devices: Vec<Device>,
}

impl NativeManager {
    pub fn new() -> NativeManager {
        let inotify = inotify_new();
        let fd = epoll_new();

        epoll_add(fd, inotify);

        let mut nm = NativeManager {
            inotify,
            fd,
            devices: Vec::new(),
        };

        // Look for joysticks immediately.
        let paths = fs::read_dir("/dev/input/by-id/");
        let paths = if let Ok(paths) = paths {
            paths
        } else {
            return nm;
        };

        for path in paths {
            let path_str = path.unwrap().path();
            let path_str = path_str.file_name().unwrap();
            let path_str = path_str.to_str().unwrap();

            // An evdev device.
            if path_str.ends_with("-event-joystick") {
                println!("js");
                let mut event = Event {
                    wd: 0,       /* Watch descriptor */
                    mask: 0x100, /* Mask describing event */
                    cookie: 0,   /* Unique cookie associating related
                                 events (for rename(2)) */
                    len: 0,         /* Size of name field */
                    name: [0; 256], /* Optional null-terminated name */
                };

                let path_str = path_str.to_string().into_bytes();

                for i in 0..path_str.len().min(255) {
                    event.name[i] = path_str[i];
                }

                inotify_read2(&mut nm, event);
            }
        }

        nm
    }

    /*    /// Do a search for controllers.  Returns number of controllers.
    pub fn search(&mut self) -> (usize, usize) {
        let devices = find_devices();

        // Add devices
        for mut i in devices {
            if self.devices.contains(&i) {
                continue;
            }

            open_joystick(&mut i);

            // Setup device for asynchronous reads
            if i.fd != -1 {
                joystick_async(i.fd);

                let index = self.add(i);
                return (self.devices.len(), index);
            }
        }

        (self.num_plugged_in(), ::std::usize::MAX)
    }*/

    pub fn get_id(&self, id: usize) -> (u32, bool) {
        if id >= self.devices.len() {
            (0, true)
        } else {
            let (a, b) = joystick_id(self.devices[id].fd);

            (a, b)
        }
    }

    pub fn get_abs(&self, id: usize) -> (i32, i32, bool) {
        if id >= self.devices.len() {
            (0, 0, true)
        } else {
            joystick_abs(self.devices[id].fd)
        }
    }

    pub fn get_fd(&self, id: usize) -> (i32, bool, bool) {
        let (_, unplug) = self.get_id(id);

        (
            self.devices[id].fd,
            unplug,
            self.devices[id].name[0] == b'\0',
        )
    }

    pub fn num_plugged_in(&self) -> usize {
        self.devices.len()
    }

    pub fn disconnect(&mut self, fd: i32) -> usize {
        for i in 0..self.devices.len() {
            if self.devices[i].fd == fd {
                epoll_del(self.fd, fd);
                joystick_drop(fd);
                self.devices[i].name[0] = b'\0';
                return i;
            }
        }

        panic!("There was no fd of {}", fd);
    }

    /*    pub(crate) fn poll_event(&self, i: usize, state: &mut State) {
        while joystick_poll_event(self.devices[i].fd, state) {}
    }*/

    /*    fn add(&mut self, device: Device) -> usize {
        let mut r = 0;

        for i in &mut self.devices {
            if i.name[ == None {
                *i = device;
                return r;
            }

            r += 1;
        }

        epoll_add(self.fd, device.fd);
        self.devices.push(device);

        r
    }*/
}
impl Drop for NativeManager {
    fn drop(&mut self) {
        while let Some(device) = self.devices.pop() {
            self.disconnect(device.fd);
        }
        unsafe {
            close(self.fd);
        }
    }
}

/*// Find the evdev device.
fn find_devices() -> Vec<Device> {
    let mut rtn = Vec::new();
    let paths = fs::read_dir("/dev/input/by-id/");
    let paths = if let Ok(paths) = paths {
        paths
    } else {
        return vec![];
    };

    for path in paths {
        let path_str = path.unwrap().path();
        let path_str = path_str.to_str().unwrap();

        // An evdev device.
        if path_str.ends_with("-event-joystick") {
            rtn.push(Device {
                name: Some(path_str.to_string()),
                fd: -1,
            });
        }
    }

    rtn
}*/

// Set up file descriptor for asynchronous reading.
fn joystick_async(fd: i32) {
    let error = unsafe { fcntl(fd, 0x4, 0x800) } == -1;

    if error {
        panic!("Joystick unplugged 2!");
    }
}

// Get the joystick id.
fn joystick_id(fd: i32) -> (u32, bool) {
    let mut a = [0u16; 4];

    extern "C" {
        fn ioctl(fd: i32, request: usize, v: *mut u16) -> i32;
    }

    if unsafe { ioctl(fd, 0x_8008_4502, &mut a[0]) } == -1 {
        return (0, true);
    }

    (((a[1] as u32) << 16) | (a[2] as u32), false)
}

fn joystick_abs(fd: i32) -> (i32, i32, bool) {
    #[derive(Debug)]
    #[repr(C)]
    struct AbsInfo {
        value: i32,
        minimum: i32,
        maximum: i32,
        fuzz: i32,
        flat: i32,
        resolution: i32,
    }

    let mut a = unsafe { mem::uninitialized() };

    extern "C" {
        fn ioctl(fd: i32, request: usize, v: *mut AbsInfo) -> i32;
    }

    if unsafe { ioctl(fd, 0x_8018_4540, &mut a) } == -1 {
        return (0, 0, true);
    }

    (a.minimum, a.maximum, false)
}

// Disconnect the joystick.
fn joystick_drop(fd: i32) {
    if unsafe { close(fd) == -1 } {
        panic!("Failed to disconnect joystick.");
    }
}

// // // // // //
//    EPOLL    //
// // // // // //

#[repr(C)]
union EpollData {
    ptr: *mut std::ffi::c_void,
    fd: i32,
    uint32: u32,
    uint64: u64,
}

#[repr(C)]
struct EpollEvent {
    events: u32,     /* Epoll events */
    data: EpollData, /* User data variable */
}

extern "C" {
    fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut EpollEvent) -> i32;
}

fn epoll_new() -> i32 {
    extern "C" {
        fn epoll_create1(flags: i32) -> i32;
    }

    let fd = unsafe { epoll_create1(0) };

    if fd == -1 {
        panic!("Couldn't create epoll!");
    }

    fd
}

fn epoll_add(epoll: i32, newfd: i32) {
    let mut event = EpollEvent {
        events: 0x001, /*EPOLLIN*/
        data: EpollData { fd: newfd },
    };

    if unsafe {
        epoll_ctl(epoll, 1 /*EPOLL_CTL_ADD*/, newfd, &mut event)
    } == -1
    {
        unsafe {
            close(newfd);
        }
        panic!("Failed to add file descriptor {} to epoll {}", newfd, epoll);
    }
}

fn epoll_del(epoll: i32, newfd: i32) {
    let mut event = EpollEvent {
        events: 0x001, /*EPOLLIN*/
        data: EpollData { fd: newfd },
    };

    if unsafe {
        epoll_ctl(epoll, 2 /*EPOLL_CTL_DEL*/, newfd, &mut event)
    } == -1
    {
        unsafe {
            close(newfd);
        }
        panic!("Failed to add file descriptor to epoll");
    }
}

pub(crate) fn epoll_wait(epoll_fd: i32) -> Option<i32> {
    extern "C" {
        fn epoll_wait(epfd: i32, events: *mut EpollEvent, maxevents: i32, timeout: i32) -> i32;
    }

    let mut events: EpollEvent = unsafe { std::mem::uninitialized() };

    if unsafe {
        epoll_wait(epoll_fd, &mut events, 1 /*MAX_EVENTS*/, -1)
    } == 1
    {
        return Some(unsafe { events.data.fd });
    } else {
        return None;
    }
}

fn inotify_new() -> i32 {
    extern "C" {
        fn inotify_init() -> i32;
        fn inotify_add_watch(fd: i32, pathname: *const u8, mask: u32) -> i32;
    }

    let fd = unsafe { inotify_init() };

    if fd == -1 {
        panic!("Couldn't create inotify (1)!");
    }

    if unsafe {
        inotify_add_watch(
            fd,
            b"/dev/input/by-id/\0".as_ptr() as *const _,
            0x00000100 | 0x00000200,
        )
    } == -1
    {
        panic!("Couldn't create inotify (2)!");
    }

    fd
}

#[repr(C)]
struct Event {
    wd: i32,   /* Watch descriptor */
    mask: u32, /* Mask describing event */
    cookie: u32, /* Unique cookie associating related
               events (for rename(2)) */
    len: u32,        /* Size of name field */
    name: [u8; 256], /* Optional null-terminated name */
}

fn inotify_read2(port: &mut NativeManager, ev: Event) -> Option<(bool, usize)> {
    let mut name = [0; 256 + 17];
    name[0] = b'/';
    name[1] = b'd';
    name[2] = b'e';
    name[3] = b'v';
    name[4] = b'/';
    name[5] = b'i';
    name[6] = b'n';
    name[7] = b'p';
    name[8] = b'u';
    name[9] = b't';
    name[10] = b'/';
    name[11] = b'b';
    name[12] = b'y';
    name[13] = b'-';
    name[14] = b'i';
    name[15] = b'd';
    name[16] = b'/';
    let mut length = 0;
    for i in 0..256 {
        name[i + 17] = ev.name[i];
        if ev.name[i] == b'\0' {
            length = i + 17;
            break;
        }
    }

    let namer = String::from_utf8_lossy(&name[0..length]);

    let mut device = Device {
        name: name,
        fd: unsafe { open(name.as_ptr() as *const _, 0) },
    };

    if namer.ends_with("-event-joystick") == false || ev.mask != 0x00000100 {
        return None;
    }

    if device.fd == -1 {
        // Avoid race condition
        std::thread::sleep(std::time::Duration::from_millis(16));
        device.fd = unsafe { open(name.as_ptr() as *const _, 0) };
        if device.fd == -1 {
            return None;
        }
    }

    joystick_async(device.fd);
    epoll_add(port.fd, device.fd);

    for i in 0..port.devices.len() {
        if port.devices[i].name[0] == b'\0' {
            port.devices[i] = device;
            return Some((true, i));
        }
    }

    port.devices.push(device);
    Some((true, port.devices.len() - 1))
}

// Read joystick add or remove event.
pub(crate) fn inotify_read(port: &mut NativeManager) -> Option<(bool, usize)> {
    extern "C" {
        fn read(fd: i32, buf: *mut Event, count: usize) -> isize;
    }

    let mut ev = unsafe { std::mem::uninitialized() };

    unsafe { read(port.inotify, &mut ev, std::mem::size_of::<Event>()) };

    inotify_read2(port, ev)
}