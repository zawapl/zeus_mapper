# Set file

- Header (20 bytes)
    - 8 bytes (Version info)
    - 4 bytes # parent episodes
    - 4 bytes # colony episodes used
    - 4 bytes # colony episodes available
- ~~[Basic Episode Data](#basic-episode-data) (fake) (20 blocks of 356 bytes each)~~
- [Real Episode Data](#real-episode-data) (14 blocks of 2032 bytes each)
- ~~Stuff (148 bytes)~~
- [Episode Mythology Data](#mythology-data) (14 blocks of 300 bytes each)
- [Event Data](#event-data) (offset 0x9c00) (14 blocks of 18600 bytes)
- ~~Stuff (225 bytes?)~~
- ~~(offset 0x49549) Apparently unused .map data ( 500000 bytes ) might be revert data for editor or something. Or might
  just be left over from when map was in the .set file.~~
- (offset 0xc3669) # events for each episode
- ~~(offset 0xc3694 to 0xcb263) 10 different blocks of data, all apparently never used~~
- (offset 0xcb264 = 832100) Parent city favor for episode (10 x 4 bytes)
- (offset 0xcc38e = 836494) colony goals (4394? bytes)
- (offset 0xccabe = 838334) parent city goals (1840? bytes)

# Sav / Map file

# Basic Episode Data
- 4 byte : 01: episode exists
- 4 bytes 00's
- 4 bytes FF's for player created adventures. Has value for included adventures, possibly sound/text lookup index?
- 4 bytes episode #
- 4 bytes FF's
- 4 bytes possibly another sound/text lookup index?
- 60 bytes 00's
- 4 bytes = next episode 03=victory, 02=parent, 01=colony, 00=nothing
- 4 bytes = type of this episode 00=start 02=parent 01=colony
- 264 bytes = 00, then Ascii name of scenario

# Real Episode Data
- 0x0000 2 bytes start date
- ~~0x0002 8 bytes (00's)~~
- 0x000a 2 bytes (# months elapsed?)
- ~~0x000c 20 bytes (00's)~~
- 0x0020 4 bytes starting cash?
- ~~0x0024 8 bytes 00's (44 total)~~
- 0x002c 16 bytes (1st is map size, rest is...?)
- ~~0x003c 64 bytes text buffer1 (apparently not used)~~
- ~~0x007c 524 bytes text buffer2 (apparently not used)~~
- 0x0288 4 bytes (atlantean= 01 00 00 00)
- 0x028c 8 bytes wolfspawnX
- 0x0294 8 bytes wolfspawnY
- 0x029c 16 bytes fishX
- 0x02ac 16 bytes fishY
- 0x02bc 16 bytes UrchinX
- 0x02cc 16 bytes UrchinY
- ~~0x02dc 12 bytes 00's~~
- 0x02e8 32 bytes invasionX
- 0x0308 32 bytes invasionY
- ~~0x0328 2 bytes ?~~
- 0x032a 2 bytes # colonies done?
- 0x032c 8 bytes gamespawnX (deer?)
- 0x0334 8 bytes gamespawnY
- ~~0x033c 76 bytes 00's~~
- ~~0x0388 4 bytes FF's~~
- 0x038c 4 bytes entry point
- 0x0390 4 bytes exit point
- 0x0394 16 bytes disasterX
- 0x03a4 16 bytes disasterY
- 0x03b4 4 bytes sea entry
- 0x03b8 4 bytes sea exit
- ~~0x03bc 40 bytes 00's~~
- 0x03e4 4 bytes (00 or 01 in .sav?)
- 0x03e8 16 bytes gamespawnX (boar?)
- 0x03f8 16 bytes gamespawnY
- 0x0408 200 bytes building flags (2 bytes each)
- 00=not active, 01 = active
- 0x04d0 16 bytes 00's
- 0x04e0 12 bytes monX
- 0x04ec 12 bytes monY
- 0x04f8 12 bytes disembark X
- 0x0504 12 bytes disembark Y
- ~~0x0510 276 bytes 00's~~
- 0x0624 12 bytes landslideX
- 0x0630 12 bytes landslideY
- ~~0x063c 8 bytes 00's~~
- 0x0644 356 bytes [Basic Episode Data](#basic-episode-data) (real) see above
- 0x07a8 20 bytes city resources
- 0x07bc 4 bytes city items bought for colony episode
- 2 bytes unused?
- 0x07c2 4 bytes for city items sold
- 2 bytes unused?
- 0x07c8 40 bytes Quantity bought/sold 0c=low (12)
- 0x18=med 0x24=high
- 0x07f0 end

# Mythology Data


# Event data