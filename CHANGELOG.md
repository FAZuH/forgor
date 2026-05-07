## 0.2.8 (2026-05-07)


### Bug Fixes

* Updating timer settings does not update timer data ([c3826c1](https://github.com/FAZuH/tomo/commit/c3826c14baf010caba6eb75b160282b724bcf8cd)), closes [#65](https://github.com/FAZuH/tomo/issues/65)


### Documentation

* Add demo video to README ([65bdeba](https://github.com/FAZuH/tomo/commit/65bdeba6e9d9eb879467a03eb3d7163afdce5c76))


### New Features

* Add path autocomplete to settings ([72d5f2e](https://github.com/FAZuH/tomo/commit/72d5f2eee11d7765d3cb1f4619a079c76250c70b)), closes [#34](https://github.com/FAZuH/tomo/issues/34)
* Show idle time in prompt transition ([3e1e8f8](https://github.com/FAZuH/tomo/commit/3e1e8f821a5196c94fbfcc682c424994684386db)), closes [#59](https://github.com/FAZuH/tomo/issues/59)

## 0.2.7 (2026-05-07)


### New Features

* Add -t/--task argument to set initial task ([a789074](https://github.com/FAZuH/tomo/commit/a789074165449dae9b72827e3a8b484314f89341)), closes [#51](https://github.com/FAZuH/tomo/issues/51)
* Add CLI argument help ([67854fc](https://github.com/FAZuH/tomo/commit/67854fc37587a4e407e3227196857c74e855810c))
* Add prompt to set tracked task in timer view ([78ae914](https://github.com/FAZuH/tomo/commit/78ae914d8d4c35c49e20ce122aee274d94b63326)), closes [#54](https://github.com/FAZuH/tomo/issues/54)
* Add task autocompletion to task change prompt ([145924b](https://github.com/FAZuH/tomo/commit/145924b24421a6fcbd02b6f2e7dd71fa1d25f7b5)), closes [#52](https://github.com/FAZuH/tomo/issues/52)
* Display currently tracked task in timer view ([78faadd](https://github.com/FAZuH/tomo/commit/78faadd1d35e9d9d2e89ad7d3e3e00c41bf9c5d1)), closes [#53](https://github.com/FAZuH/tomo/issues/53)


### Bug Fixes

* -c/--config-path not working properly ([ad036f2](https://github.com/FAZuH/tomo/commit/ad036f29797de6623622daef5bd05b7604b1dbc0))

## 0.2.6 (2026-05-06)


### New Features

* Add confirmation prompt when resetting timer ([d2dc423](https://github.com/FAZuH/tomo/commit/d2dc423bff92191aa4cf7cc609cd9bb43dfcc251)), closes [#56](https://github.com/FAZuH/tomo/issues/56)
* Add warning when quitting with unsaved settings changes ([f63dec3](https://github.com/FAZuH/tomo/commit/f63dec3e5a4efe8cda817f2aeef4932ec8ab5368)), closes [#37](https://github.com/FAZuH/tomo/issues/37)


### Bug Fixes

* Frame not redrawing when typing on settings ([7f812bf](https://github.com/FAZuH/tomo/commit/7f812bffd0c71bb9904018db621c815ffc92be50))

## 0.2.5 (2026-05-06)


### Bug Fixes

* Transition prompt not disappearing after submitting ([65b7597](https://github.com/FAZuH/tomo/commit/65b75979bf49b93d04fd8ea4249820788bcc1bc1))


### New Features

* Add warning when trying to start the program when another ([ca981c1](https://github.com/FAZuH/tomo/commit/ca981c1094b1d97b608d46d7364a1879e1839b83)), closes [#40](https://github.com/FAZuH/tomo/issues/40)


### UI Changes

* Add settings item descriptions ([2be2907](https://github.com/FAZuH/tomo/commit/2be2907844c307484c16bbf07f6efe4f76056620)), closes [#35](https://github.com/FAZuH/tomo/issues/35)


### Performance Improvements

* Redraw frame on resize ([50f035d](https://github.com/FAZuH/tomo/commit/50f035ddb50ff55b737f86411643647c26d98c86)), closes [#36](https://github.com/FAZuH/tomo/issues/36)

## 0.2.4 (2026-05-05)


### New Features

* Add -c/--config-path argument ([bb51ecd](https://github.com/FAZuH/tomo/commit/bb51ecddfb1f84ba250766ed70f06261051577eb))
* Add session tracking ([c7f7a97](https://github.com/FAZuH/tomo/commit/c7f7a97ae0977f8fc411a437f67b279aee34c1f6))
* Add settings copy and paste keybinds ([c2fb9b5](https://github.com/FAZuH/tomo/commit/c2fb9b52b3398ecb8f231c559a8d9f5a80e60b51)), closes [#33](https://github.com/FAZuH/tomo/issues/33)
* Add stop alarm keybind ([110c33b](https://github.com/FAZuH/tomo/commit/110c33b9171c144fd0c58f5bdb7f89b41dcf1a2c))
* Add stop alarm keybind hint ([77ea64c](https://github.com/FAZuH/tomo/commit/77ea64c26c44f1ea7022ee2e02582b668e2d87d8))
* Improve crash handling ([f4013e7](https://github.com/FAZuH/tomo/commit/f4013e7d38ee2cda5ae5be20c72ff1435d4dcba4))
* Improve crash handling ([18a7d1a](https://github.com/FAZuH/tomo/commit/18a7d1a633d319c40ee905d391a2c8589546851c))
* Show paused duration ([5f975d7](https://github.com/FAZuH/tomo/commit/5f975d729b00a773c0493e3eeb0b12946ffcf8a8)), closes [#38](https://github.com/FAZuH/tomo/issues/38)
* Show remaining focus sessions before a long break ([dcd4cd3](https://github.com/FAZuH/tomo/commit/dcd4cd3927332da81b0028826ea96ccc1da2f8a2)), closes [#39](https://github.com/FAZuH/tomo/issues/39)

