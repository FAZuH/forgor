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

## 0.2.3 (2026-05-01)


### New Features

* Add "Auto-start on Launch" setting ([0caebea](https://github.com/FAZuH/tomo/commit/0caebea06b810fc5dabdeb3c8ee7db4cb96e5537))
* Improve settings prompt titles ([15310dc](https://github.com/FAZuH/tomo/commit/15310dcf8d262c68d9d81b31455ed5202c628296))
* Improve timer transition prompt UI ([f319797](https://github.com/FAZuH/tomo/commit/f319797656557eb68aa59a54f40f3e64a93abcae))
* Make invalid path red in settings ([3983de9](https://github.com/FAZuH/tomo/commit/3983de9e70eea01ed0d717ac0c11ef3380e88d70))
* Remove duplicate keymap hint ([caa1935](https://github.com/FAZuH/tomo/commit/caa19357ffaf5fa3f2fd7b9ed09e0e603e4790b2))
* Set default alarm volume to 50% ([35031fd](https://github.com/FAZuH/tomo/commit/35031fd599a9d8b7531888565569a04643d883f0))
* Set default launch auto-start to true ([d64b643](https://github.com/FAZuH/tomo/commit/d64b643c5e6ad3dd74d44c170d8e1053fe7fa491))
* Stop alarm after transition ([1f5d2f1](https://github.com/FAZuH/tomo/commit/1f5d2f12e52a8b28f37ad9763c7b6f32578c567d))
* Transition & pause when timer prompt is answered no ([3bd3e3e](https://github.com/FAZuH/tomo/commit/3bd3e3edd7130600134cfffceb072ebe47c02959))


### Bug Fixes

* Fix alarm path change not taking effect without restart ([accd464](https://github.com/FAZuH/tomo/commit/accd464b1edee8bd66d43dd118e1389af29fb274))
* Timer keybind help symbols not rendering properly on Windows ([7b95e1c](https://github.com/FAZuH/tomo/commit/7b95e1c630fd07065d55a2481265e6b9cab2a2ac))

## 0.2.2 (2026-05-01)


### Bug Fixes

* Duplicate input on Windows command prompt ([f34a18e](https://github.com/FAZuH/tomo/commit/f34a18e5dcd0db09122c0cdea528c5fc6cdcf629))


### Performance Improvements

* Improve initial draw speed ([99c9bc5](https://github.com/FAZuH/tomo/commit/99c9bc5e7e3858a36bf1c0150ecff9636a5adea0))

## 0.2.1 (2026-05-01)


### Performance Improvements

* Fix tick timer bug causing high CPU usage ([d36489e](https://github.com/FAZuH/tomo/commit/d36489e37b93fd0b43ff85dd1143152cb3d5721a))
* Improve idle CPU usage ([f74a09f](https://github.com/FAZuH/tomo/commit/f74a09f6a99d6896b060d000572e8e9b84454c46))
* Redraw only when a valid input is pressed ([50c0e1d](https://github.com/FAZuH/tomo/commit/50c0e1ddc46f8e67129886b3298c6151ced8b3da))

## 0.2.0 (2026-05-01)


### Bug Fixes

* Alarm volume settings showing alarm path when editing ([1a389a9](https://github.com/FAZuH/tomo/commit/1a389a914127bc3cb32a387ae67c4148428d1363))
* Crash when toast exceeds frame height ([1de1a29](https://github.com/FAZuH/tomo/commit/1de1a2942819df43eb8e4cbc9d753786a333dc40))
* Fix toast deduplication issues ([8c098d2](https://github.com/FAZuH/tomo/commit/8c098d2e53ca6ded0257bb8f42301e7f445da3df))


### New Features

* Add settings page keybind help ([8db8c50](https://github.com/FAZuH/tomo/commit/8db8c501ad92318257e2465e151a70a6902b585b))
* Add settings section navigation and improve UI ([f628c27](https://github.com/FAZuH/tomo/commit/f628c271fd20d0cbd0adb65dc27fcfb280499a6a))
* Add settings section select buttons ([3124353](https://github.com/FAZuH/tomo/commit/312435375e4b7db62bbef1c89a294c874f6005c2))
* Add toast deduplication ([eaab8f0](https://github.com/FAZuH/tomo/commit/eaab8f0da4fe7371675d8b71d847a1bc594b823d))
* Adjust padding of settings ([33fd694](https://github.com/FAZuH/tomo/commit/33fd69471d68587fd100028cd9c83cbbbac384e8))
* Improve settings layout ([4024c54](https://github.com/FAZuH/tomo/commit/4024c544edb3b11bc822eb90e2fd42ff9ddf8ae4))
* Improve timer page keybind hint ([8ffca11](https://github.com/FAZuH/tomo/commit/8ffca117a355b311921ca9aa06a856b12eb7b34c))
* Invert timer 30sec offset keybinds ([d586abd](https://github.com/FAZuH/tomo/commit/d586abd445ebd8d8dbc318bee4872760e7da8750))
* Make settings checkbox label dim ([08f8dcd](https://github.com/FAZuH/tomo/commit/08f8dcdf3b65a07c54b889f2e6bee24b33b1aa92))
* Make timer shortcut toggleable ([8900478](https://github.com/FAZuH/tomo/commit/8900478530b941b3d986099d914814c3bd5784b4))
* Trim percent when editing alarm volume ([83a47ec](https://github.com/FAZuH/tomo/commit/83a47ecee1437bec36c7bad6229306bd8a636bb0))

