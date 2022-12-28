## Packaging

# v3.0.3

https://github.com/proshunsuke/colmsg/pull/88

## Fix

* Use [rayon](https://crates.io/crates/rayon) to speed up internal processing

# v3.0.2

https://github.com/proshunsuke/colmsg/pull/86

## Fix bugs

* fixed error when files other than message files exist in the download directory

# v3.0.1

https://github.com/proshunsuke/colmsg/pull/80

## Fix bugs

* support OpenSSL 3

# v3.0.0

https://github.com/proshunsuke/colmsg/pull/68

## BREAKING CHANGES

* support "乃木坂46メッセージ"
* save past messages which delivered up to 24 hours before subscription start
* `-g` option allow multiple
* do not save if a refresh token is not specified for a group

## Changes

* improve development
  * create OpenAPI specifications
  * create docker environment to launch mock servers
* fix README for "乃木坂46メッセージ"
* fix doc for "乃木坂46メッセージ"

# v2.0.4

https://github.com/proshunsuke/colmsg/pull/57

## Fix bugs

* fixed an issue that could not be saved messages because of missing member_id

# v2.0.3

https://github.com/proshunsuke/colmsg/pull/51

## Fix bugs

* changed api version with the version up of app

# v2.0.2

https://github.com/proshunsuke/colmsg/pull/48

## Fix bugs

* fixed issue that can not save hinatazaka messages

# v2.0.1

https://github.com/proshunsuke/colmsg/pull/44

## Fix bugs

* write standard error output when sakurazaka
* fix letters error

# v2.0.0

https://github.com/proshunsuke/colmsg/pull/37

## BREAKING CHANGES

* support "櫻坂46メッセージ" and "日向坂46メッセージ"
* **no longer support "欅坂46/日向坂46 メッセージ"**
* add --s_refresh_token and --h_refresh_token options
* remove --refresh_token option

## Changes

* update README.md
* update doc
* add doc for "欅坂46/日向坂46 メッセージ" users

# v1.0.0

https://github.com/proshunsuke/colmsg/pull/30

## Changes

* support "欅坂46/日向坂46 メッセージ" version 2.0.00
* support Windows
* update README.md
* update doc
  * change how to get token
* change interface for "欅坂46/日向坂46 メッセージ" version 2.0.00
  * run "colmsg --help" for details

# v0.1.3

https://github.com/proshunsuke/colmsg/pull/26

## Fix bugs
   
* fix nullable parameters

# v0.1.2

https://github.com/proshunsuke/colmsg/pull/22

## Fix
   
* print error details when post request

# v0.1.1

https://github.com/proshunsuke/colmsg/pull/17

## Fix bugs
   
* infinite loop when saving messages
* crash when letter message

# v0.1.0

Initial release
