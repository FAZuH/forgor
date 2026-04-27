## [0.1.6](https://github.com/FAZuH/tomo/compare/v0.1.5...v0.1.6) (2026-04-27)


### refactor

* **core:** Set default long interval from 3 to 4 ([6bc3250](https://github.com/FAZuH/tomo/commit/6bc3250f2e400c551df90abb4eeb4eb9ea2b1abc))


### feat

* **core:** Add command hooks on session end ([71907e7](https://github.com/FAZuH/tomo/commit/71907e74ac78fba5079f3a8750f972bfb65d2b6d)), closes [#4](https://github.com/FAZuH/tomo/issues/4)
* **core:** Notify on session transitions ([45b8e90](https://github.com/FAZuH/tomo/commit/45b8e90deea6c34306fcea34ae653614dd25d500))

## [0.1.5](https://github.com/FAZuH/tomo/compare/v0.1.4...v0.1.5) (2026-04-27)


### ⚠ BREAKING CHANGES

* Rename "notification" to "alarm" [pub] [no ci]
* Add notification sound when a session finishes [pub]

* refactor!(core): Rename "sound" to "notification" ([98a4b75](https://github.com/FAZuH/tomo/commit/98a4b750e9051806ac1a23e4a919d4d33f3638e5))
* feat!(core): Add notification volume to config ([2361c69](https://github.com/FAZuH/tomo/commit/2361c69c23ee435a645532b3d5d3c95f29000aee))


### feat

* **tui:** Add notification volume configuration to settings page ([747c9a6](https://github.com/FAZuH/tomo/commit/747c9a69bd76db45aea15a9ebae351eec9779ed6))
* **tui:** Add prompt for next session when session ends ([87889f0](https://github.com/FAZuH/tomo/commit/87889f04ddaa7013ac392b56daeca1d058972834))


### perf

* **tui:** Minor performance improvement in timer page rendering ([bd2b171](https://github.com/FAZuH/tomo/commit/bd2b171a564e4e57169ef1f6244388d67dbcd6a1))


### fix

* **tui:** Fix pomodoro state label not centered when paused ([44bbdfe](https://github.com/FAZuH/tomo/commit/44bbdfef3f73426b62bf46fbc317ed12b5af2eb3))


### New Features

* Add notification sound when a session finishes ([8cc366d](https://github.com/FAZuH/tomo/commit/8cc366dc2dfe180e4a994ce7da68eec9b673e35c))


### Code Refactoring

* Rename "notification" to "alarm" ([d923572](https://github.com/FAZuH/tomo/commit/d923572e9de6db6530cee892be9e2964a26c90cf))

## [0.1.4](https://github.com/FAZuH/tomo/compare/v0.1.3...v0.1.4) (2026-04-26)


### feat

* **tui:** Add settings save by 's' shortcut ([9bd3f51](https://github.com/FAZuH/tomo/commit/9bd3f512c13c6683410537cd1a50b5555090a5d3))
* **tui:** Add unsaved changes indicator to settings page ([a2b65cd](https://github.com/FAZuH/tomo/commit/a2b65cd145914181db92c7c84debf6cd6bf9ff8d))


### refactor

* Rename logging var to TOMO_LOG ([d825181](https://github.com/FAZuH/tomo/commit/d825181c519b8b29b5834863f90ae1e689fde568))

## [0.1.3](https://github.com/FAZuH/tomo/compare/v0.1.2...v0.1.3) (2026-04-25)


### feat

* **db:** Add database ([5aab18d](https://github.com/FAZuH/tomo/commit/5aab18dbe807bd615d3f0962fd97c9f7d53d38ca))
* **tui:** Improve settings UI ([86c0bf8](https://github.com/FAZuH/tomo/commit/86c0bf8223fc15d1a9f4139041090fa28638ca8c))


### docs

* Add screenshot to README ([8d8c699](https://github.com/FAZuH/tomo/commit/8d8c69945471185d75985eaaf80c6b46e449dc8a))


### fix

* Duration configs not saved as simple seconds ([796835f](https://github.com/FAZuH/tomo/commit/796835ff3cb929a9c520424525915688292f1d0f))

## [0.1.2](https://github.com/FAZuH/tomo/compare/v0.1.1...v0.1.2) (2026-04-17)


### fix

* config not being loaded properly ([5ec5829](https://github.com/FAZuH/tomo/commit/5ec58299f87469f63c6f986ac417d8d75322810a))


### feat

* **tui:** Add vim-like bindings for timer adjustment ([1206f1e](https://github.com/FAZuH/tomo/commit/1206f1e92bf6fc9963b518dc97abd07b8ea064b9))
* **tui:** toggle settings checkbox space/enter ([a730298](https://github.com/FAZuH/tomo/commit/a730298e4c3d477289c3520998b72fac21d5b278))


### perf

* **tui:** Improve CPU usage ([8900d8c](https://github.com/FAZuH/tomo/commit/8900d8c783fdc5013bcbbbfd69ea6b6ef82c427f))

