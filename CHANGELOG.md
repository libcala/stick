# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://free.plopgrizzly.com/semver/).

## [Unreleased]
### Added
- `poll()` method to block until an event is received from any controller.
### Removed
- Pan separate from camera Y.
### Changed
### Fixed

## [0.6.0] - 2019-05-13
### Added
- `Device.lrt()` function to get left & right trigger values.

### Fixed
- Can only extract `Device.joy()` values once.

## [0.5.0] - 2019-05-12
### Added
- Full support for 4 joysticks
- New API with `Port`, `Device` and `Btn`
- API to detect whether or not joystick features are supported (not complete).

### Removed
- `ControllerManager` & `Input`

### Changed
- Input is now received through function calls like `joy()` instead of the `Input` Enum

## [0.4.1] - 2018-08-05
### Fixed
- Crash on specific hardware running Linux.

## [0.4.0] - 2018-05-23
### Added
- Fake Windows support.

### Removed
- `Button` - Now incorporated into `Input`.

## [0.3.0] - 2018-02-03
### Added
- Added `ControllerManager` to simplify API

### Removed
- Removed `Throttle` struct and `Joystick` struct

## [0.2.0] - 2018-01-27
### Added
- Remapping

### Changed
- Use evdev instead of js0

## [0.1.0] - 2018-01-01
### Added
- Linux Support