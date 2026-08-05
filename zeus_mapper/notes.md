from: https://caesar3.heavengames.com/cgi-bin/caeforumscgi/display.cgi?action=st&fn=15&tn=7298

Ah, at least 1 interested party then, good to know. Have been distracted, not paying attention to this forum. Sorry for the slow reply.

I hadn't actually thought along the lines of sharing .exe file info, though I have reverse engineered/decompiled enough of the program that I could in theory produce fixes for the 80+ bugs/issues I've identified. I might even someday produce a patch. Or not.

As far as hex editing of .map/.set/.pak/.sav files, I'm actually not sure what to share and a lot of my notes are a mess, but I'll try to post some stuff out when I have more time in a day or 2.

ADDED 03-24:

The .set file is entirely uncompressed data and has a rigid format:
```
Header (20 bytes)
Basic Episode Data (fake) (20 blocks of 356 bytes each)
Real Episode Data (14 blocks of 2032 bytes each)
Stuff (148 bytes)
Episode Mythology Data (14 blocks of 300 bytes each)
Event Data (offset 0x9c00) (14 blocks of 18600 bytes)
Stuff (225 bytes?)
(offset 0x49549) Apparently unused .map data ( 500000 bytes ) might be revert data
for editor or something. Or might just be left over from when
map was in the .set file.
(offset 0xc3669) # events for each episode
(offset 0xc3694 to 0xcb263) 10 different blocks of data, all apparently never used
(offset 0xcb264) (Parent city favor for episode) 10 x 4 bytes

(offset 0xcc38e) colony goals
(offset 0xccabe) parent city goals
```

---

.sav and .map files are not of fixed length. However, the 1st 8(save) or 12(map) bytes are version information, and the next 6000 bytes are a manifest of sorts. Each element in the manifest is 20 bytes:
```
4 bytes 01=compressed 00=not compressed
4 bytes memory address data came from
4 bytes sizeMult1
4 bytes sizeMult2
4 bytes ?(not important)
```

Modifying the manifest does nothing, the game just checks the file's version number and then assumes that it is in the correct format for that version. If the file turns out to not be in the correct format because some numbskull was playing around with a hex editor, the game will helpfully crash and burn.

If all the blocks were uncompressed, the .sav and .map files would have a fixed length--A very large fixed length, about 4MB for each save, rather than the 300-500K it usually is.

---

For uncompressed data, the actual size of the data in the file would be (sizeMult1 x sizeMult2)

Compressed data blocks consist of:
```
4 bytes for the of the block
2 bytes with the values: 00 06
(size - 2) bytes of data
```

does not include itself, but does include the "00 06", So total number of bytes used would be +4.

`99 30 00 00 00 06`
would indicate a compressed block of = 0x3099 (12441) bytes.

One exception to this is that a value of "00 00 00 80" means that the compressed data block is in fact uncompressed, because no useful compression could be applied. If I recall correctly, this means that the next 51984 bytes will be considered the data for that block. If the block is supposed to be smaller, the rest of the data in the file will be unused. I'm not sure what happens if the block is supposed to be larger, but I recall having trouble inserting an uncompressed terrain block (which is 200K) into a map file.

---



An example of the use of this is to locate the Real Episode data in a sav file. We know it is 2032 bytes, so we search for or "f0 07 00 00", which is hexidecimal for 2032(7f0), written in little-endian order(used by programs written for intel chips). We come up with:

```00000000 98189000 f0070000 01000000 00000000```

This shows that it is the 10th entry in the manifest. Adding up the values of the sizes of things, we'd find that we need to wander down 6009 bytes, then a compressed block, 8 bytes, another compressed block, then 144 bytes, which would bring us to the data for the current episode.

---

.Set file Header: (20 bytes total) 0x0000 to 0x0013
```
8 bytes (Version info)
4 bytes # parent episodes
4 bytes # colony episodes used
4 bytes # colony episodes available
```

---

Basic Episode data (fake): note that this is a copy of part of Real Episode Data
This appears to be unused. Changing it will do nothing.

```
4 byte : 01: episode exists
4 bytes 00's
4 bytes FF's for player created adventures. Has value
for included adventures, possibly sound/text lookup index?
4 bytes episode #
4 bytes FF's
4 bytes possibly another sound/text lookup index?
60 bytes 00's
4 bytes = next episode 03=victory, 02=parent, 01=colony, 00=nothing
4 bytes = type of this episode 00=start 02=parent 01=colony
264 bytes = 00, then Ascii name of scenario
```

---

Real Episode data :

Notes:
- a single such block exists in .sav, and .map files
- a .set file has 10 for parent episodes and 4 for colonies
- not all bytes will be used in a particular file type
  for example, game points would be ignored if not in a .map file
- some bytes would only have meaning for the 1st episode, whose
  value would apply to all episodes

Offset
```
0x0000 2 bytes start date
0x0002 8 bytes (00's)
0x000a 2 bytes (# months elapsed?)
0x000c 20 bytes (00's)
0x0020 4 bytes starting cash?
0x0024 8 bytes 00's (44 total)
0x002c 16 bytes (1st is map size, rest is...?)
0x003c 64 bytes text buffer1 (apparently not used)
0x007c 524 bytes text buffer2 (apparently not used)
0x0288 4 bytes (atlantean= 01 00 00 00)
0x028c 8 bytes wolfspawnX
0x0294 8 bytes wolfspawnY
0x029c 16 bytes fishX
0x02ac 16 bytes fishY
0x02bc 16 bytes UrchinX
0x02cc 16 bytes UrchinY
0x02dc 12 bytes 00's
0x02e8 32 bytes invasionX
0x0308 32 bytes invasionY
0x0328 2 bytes ?
0x032a 2 bytes # colonies done?
0x032c 8 bytes gamespawnX (deer?)
0x0334 8 bytes gamespawnY
0x033c 76 bytes 00's
0x0388 4 bytes FF's
0x038c 4 bytes entry point
0x0390 4 bytes exit point
0x0394 16 bytes disasterX
0x03a4 16 bytes disasterY
0x03b4 4 bytes sea entry
0x03b8 4 bytes sea exit
0x03bc 40 bytes 00's
0x03e4 4 bytes (00 or 01 in .sav?)
0x03e8 16 bytes gamespawnX (boar?)
0x03f8 16 bytes gamespawnY
0x0408 200 bytes building flags (2 bytes each)
00=not active, 01 = active
0x04d0 16 bytes 00's
0x04e0 12 bytes monX
0x04ec 12 bytes monY
0x04f8 12 bytes disembark X
0x0504 12 bytes disembark Y
0x0510 276 bytes 00's
0x0624 12 bytes landslideX
0x0630 12 bytes landslideY
0x063c 8 bytes 00's
0x0644 356 bytes Basic Episode Data (real) see above
0x07a8 20 bytes city resources
0x07bc 4 bytes city items bought for colony episode
2 bytes unused?
0x07c2 4 bytes for city items sold
2 bytes unused?
0x07c8 40 bytes Quantity bought/sold 0c=low (12)
0x18=med 0x24=high
0x07f0 end
```

---

MYTHOLOGY
```
12 opponent gods (4 prox, 8 distant) (48 bytes) ffffffff = none
12 proponent gods (6 prox, 6 distant) (48 bytes)
Independent monster (4 bytes )
(96 bytes) FF's (aparently unused)(maybe for distant gods, originally 12 prox gods intended?)
(12 bytes) 00's (guess = unused sanctuary to opponent gods?)
Sanc available. (12 bytes) 01= can build sanc, 00=can't. Only pays attention to 1st 6
4 bytes # sanctuaries allowed
4 bytes # pyramids allowed
72 bytes pyramid data
(4 bytes type, 4 bytes deity, 4 bytes coloration) for each pyramid
```

---


EVENTS

124 bytes per event

```
9C00
9C7B last byte of1st event
```

EVENT SUBTYPE:  (offset 0x60)

Monster invasion:
```
00=monster in city    01=monster unleashed        02=monter invades
```

Request:
```
00 general request         01 under threat of invasion
02 attacking rival         03 festival
04 construction            05 famine
06 financial woes          07 city menaced by monster
08 ?                       09 player attack rival  <yes, this is a "request" type event...>
```

City status change:
```
00 trade suspended (they're provoked)         01 trade resumes
02 trade shuts down (no given reason)         03 trade opens up
04 tribute suspended                          05 tribute resumed
06 rebellion                                  07 rebellion over?
08 colony becomes rebellious                  09 rival becomes ally                         
0a ally becomes rival                         0b city becomes vassal
0c ally becomes rival                         0d god disaster
0e military buildup                           0f military decline
10 econ increase                              11 economic decline
12 city becomes active                        13 city becomes inactive
14 city appears                               15 city disappear
16 god disaster over                          17 rebellion over
18 city conquered
```

(1F) opionion expression event:
```
01 = hostile(rival)     02 = angry(ally?)     03 = colony                04 = ?               05 = parent
06 = respect(rival)     07 = loves(           08 = COLONY love           09 = parent love
```

COMMODITIES:
```
01=urchin     02=fish       03=meat        04=cheese
05=carrots    06=onions     07=wheat       08=oranges
09=wood       0a=bronze     0b=marble      0c=grapes
0d=olives     0e=fleece     0f=horses      10=black marble
11=orihalc    12=armor      13=sculpture   14=olive oil
15=wine       16=chariots   17=drachma     18=troops
19=food       1A=Hero       1B=?
```

00 or ff=none


MYTH TYPE
```
00= monster1     01=monster2    02=independent monster

00= hero1  01=hero2 02=hero3

00=zeus      01=poseidon     02=demeter      03=apollo
04=artemis   05=ares         06=aphrodite    07=hermes
08=Athena    09=hephaestus   0a=dionysus     0b=hades
0c=hera      0d=atlas
```

Table indicates offset from start of event data.
Format: offset XY
```
   | X0    X1    X2    X3    | X4    X5    X6    X7    | X8    X9    Xa    Xb    | Xc    Xd    Xe    Xf    |
   |_________________________|_________________________|_________________________|_________________________|
0Y | EventID#  | Type |Month | ItemChosen |FirstItem   | SecondItem  ThirdItem   | AmountChose FixedAmount |
   |           |      |      |            |            | (00)  (00)  (00)  (00)  |                         |
   |_________________________|_________________________|_________________________|_________________________|
1Y | MinAmount   MaxAmount   | TimeChose   FixedTime   | MinTime     MaxTime     | TargetChose FixedTarget |
   | (05)        (0A)        |                         | (01)        (02)        |                         |
   |_________________________|_________________________|_________________________|_________________________|
2Y | MinTarget   MaxTarget   | EvtToTrig   EvtToTrig   | Recur/other flags       | Warning     TimeCtr     |
   | (01)        (08)        | on Success   on Refuse  |                         | (02)                    |
   |_________________________|_________________________|_________________________|_________________________|
3Y | Status      Needmsg ?   | Triggerer   God/MonID   | Mtar1 Monu  Mtar2       | Mtar3       Magg        |
   |             res         | (FF)  (FF)  or#Warship  |                         |                         |
   |_________________________|_________________________|_________________________|_________________________|
4Y | ??                      | ??                      | ??                      | ??                      |
   | (00)  (00)  (00)  (00)  | (00)  (00)  (00)  (00)  | (00)  (00)  (00)  (00)  | (00)  (00)  (00)  (00)  |
   |_________________________|_________________________|_________________________|_________________________|
5Y | ???   ???   EvToTrigOn  | EvToTrigOn  EffOnCity   | SourceChose SourceFixed | SourceMax   SourceMax   |
   |               ????      |   ????                  |                         | (01)        (08)        |
   |_________________________|_________________________|_________________________|_________________________|
6Y | Subtype     PrevAmt?    | ???   ???   ???   ???   | ???   ???   TrigReason  | ???   ???   ???   Other |
   |                         | (related to trigged evt)|                         |                   City  |
   |_________________________|_________________________|_________________________|_________________________|
7Y | loot       Loot         | ???  Ally  Ally   Tot   | ???   ???   Which  ???  |
   | Type        Amt         |      City  Str    Str   |             Quest       |                         
   |_________________________|_________________________|_________________________|
```


Event Types:
```
00=free event          01=troops/goods request             02=invasion                 03=earthquake          
04=quest               05=landslide                        06=sea trade problem(*)     07=land trade problem(*)
08=wage increase       09=wage decrease                    0a=contaminated water(*)    0b=copper mine collapse(*)
0c=clay pit flood(*)   0d=demand increase                  0e=demand decrease          0f=price increase
10=price decrease      11=favor increase                   12=favor decrease           13=city status change(trade shuts down, other?)
14=rival army away     15=trade change(increase)           16=trade change(decrease)   17=gift
18=lava flow           19=disaster(tidal wave)             1a=monster unleashed        1b=god invasion
1C=sink land           1d=Tribute Granted(A)               1e=rival demands tribute? (A)
1F=opinion change  (A) 20=Request Granted by AI (A)        21=request refused (A)      22=ally mounts attack for player (A)
23=olympics start (A)  24=no competitors for olympics (A)  25=Olympics over (A)        26=God blessing (A)
27=hero summoned (A)   28-2c= <blank event>                2d+=<kaboom>!
```

(*)  Appears to be unused
   A  Event normally created automatically by computer


HEADER:
```
(00)(01) event ID# 00-??
(02) event type
(03) month of event occurance
```

ITEM
```
(04)(05) actual item chosen from the 3 items (item is god, commodity,...)
(06)(07) first possible item
(08)(09) second possible item
(0A)(0B) third possible item
```

QUANTITY
```
(0C)(0D)(0E)(0F)(10)(11)(12)(13) RANGE/VC QUANTITY of goods, invaders, etc.

(14)(15)(16)(17)(18)(19)(1A)(1B) RANGE/VC TIME till begin. Years from episode start for scheduled events, months from trigger for triggered events.

(1C)(1D)(1E)(1F)(20)(21)(22)(23) RANGE/VC: CITY (request) or marker(invasion)
```

TRIGGER
```
(24)(25) FF FF normal or XX 00 : trigger this eventID on SUCCESS
(26)(27) FF FF normal : trigger this eventID on REFUSAL
```

OCCURANCE
```
(28)(29)
(2A)(2B)
```
```
XX .. .. ..  0x000000XX  1:you weak 2:CPU created?  4:oracle 8:oracle did | 1:triggered only  2:recurring 4:recur event complete, mil event complete 8:is triggered
.. XX .. ..  0x0000XX00  4:raiding party returning 2:involves hero(?) 1:ares came along(?)| 1:finish up at end of scenario
.. .. XX ..  0x00XX0000  | 1:visible on map? 2:do at end of episode  4:set when close to arrival for invasion? 8:won?
.. .. .. XX  0xXX000000
```

```
(2C)(2D) # months warning before event
(2E)(2F) months used/left/whatever

(30)(31) Event status from 00 to 0a

(32) flag:waiting for dialog response?
(33) flag:request placed on request list?


(34)(35) Triggering event ID#? FF FF
(36) godID, or monsterSlot, or HeroSlot # warships for naval invasion
(37) (37 gets 36 from trigger event)

(38) monster target
(39) 01=small monument 02 =large monument
(3A)(3B) (monster invasion) monster target
(3C)(3D) (monster invasion) monster target
(3E)(3F) (monster invasion) aggressivness of monster
```

Monster targets:
```
00=food 01=sea 02=industry 03=military 04=money 05=troops
06=common 07=aesthetic 08=mythological 09=best 0A=random
```

This can be set for regular invaders too, but as far as I can tell, they ignore it, although the unit data has a field that receives the value, so they ought to do something :P.

```
(52)(53) FF FF Triggered event on LATE
(54)(55) FF FF Triggered on responded but LOST

(56) 01=monster/attacker will destroy city 02=will conquer city
(57) 01=1st warning given. 11=final warning before arrival given? 04 = can't invade (announce invasion?) till this flag is set...

(58)(59)(5A)(5B)(5C)(5D)(5E)(5F) RANGE/VC: city that launched invasion.

(60) SUBTYPE of event (which city status change, which kind of request, etc., which god sent us on quest, see above for options)
(61)
(62) 06 (monster invasion in progress)
(63) 00 (req response code uses (63)) (63 gets 1c (target) from previous event/event template.)

(64)(65) 00 00 (reason phrase?)
(66)(67) 00 00 (commodity from trigger event?)

(68)(69) 00 00
(6A)(6B) 00 00= Direct result 01 00 = BTW(incidental) 02 00 = despite(inspite of) 03 00=no cause 04 00=continuous/cyclical 05 00=specific as needed 06 00=auto

(6C)(6D) 00 00
(6E) 00
(6F) Secondary city ID. For Ally attacks rival, ally under attack by rival, or ally conquered

(70)(71) 00 00
(72)(73) 00 00

(74) 00
(75) 00 City allied troops came from.
(76) 00 # allied troops
(77) 00 alleged force strength of our troops.

(78)
(79) 01 : permanent flood
(7A)(7B) 01 00=2nd quest 00 00=first quest
```

RANGE/VC: 8 bytes:
```
(00) (01): actual value chosen
(02) (03): fixed value
(04) (05): range min
(06) (07): range max
```
fixed value/range will be used to calculate actual value. if one is FFFF (-1), other will be used. I forget which has priority.

---

from https://caesar3.heavengames.com/cgi-bin/caeforumscgi/display.cgi?action=ct&f=15,7292,,365:
A hex editor is actually a lot like a text editor/word processor. Except that you are viewing/editing files as hexidecimal numbers, which lets you see/modify pretty much any file on your computer. It also may show things in text, if the file contains text elements.

If I understand correctly, you are wanting to convert a .pak file into a .set file and some .map files? This is very very simple with a hex editor. All you need to understand is load/save, cut/copy/paste/delete, and find/search, all of which you've probably done before unless you have never used a word processor.

So, if you haven't already, find yourself a hex editor. I use one called "Frhed". It has a few quirks, but it's free.

First a bit of info about a .pak file:
A .set file is always 842931 bytes. The first 842931 bytes of the .pak file are the data from the .set file. Somewhere within those 842931 bytes will be the word "MAPS". This is probably left over from an earlier version of the game when they put the map data in the .set file, so ignore it. The second time "MAPS" shows up in the file will be at byte 842932 and this will be the start of the actual parent city .map data. If there are no colonies, the map data goes to the end of the file. Otherwise, the 3rd "MAPS" will be the start of the first colony, the 4th "MAPS" will be the 2nd colony, etc.

Okay, so

1. Load the .pak file into the hex editor.
2. Find the 2nd time "MAPS" appears in the file.
3. Use your editing skills to put everything before the "M" in "MAPS" into a file called (Adventure name).set.
4. Then put those the "MAPS" and everything after them till the 3rd "MAPS" (or end of file) in a file called (Adventure name)P.map.
5. If there are colonies, put the maps in appropriately named colony files. 1st colony is 3rd MAPS till before 4th, 2nd colony is 4th MAPS till just before 5th, etc.

Here's what the area near the "MAPS" looks like in a hex editor:
```
0cdc80 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................................
0cdca0 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 4d 41 50 53 41 01 00 00 21 00 00 00 00  ...................MAPSA...!....
0cdcc0 00 00 00 44 bd 63 00 04 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00 b0 c8 f9 00 04 00 00 00 01  ...D½c.................°Èù......
0cdce0 00 00 00 00 00 00 00 00 00 00 00 b4 c8 f9 00 04 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00 40  ...........´Èù.................@
0cdd00 b1 f9 00 14 00 00 00 2c 01 00 00 00 00 00 00 01 00 00 00 38 bf d4 00 04 00 00 00 10 cb 00 00 00  ±ù.....,...........8¿Ô......Ë...
0cdd20 00 00 00 01 00 00 00 28 f4 d3 00 01 00 00 00 10 cb 00 00 00 00 00 00 01 00 00 00 98 d0 cc 00 04  .......(ôÓ......Ë...........ÐÌ..
```

The leftmost column is the number of bytes into the file, in hex. For example, if you put "0cdca0" into a scientific calculator (windows has 1 under accessories), you get 842912. Each pair of characters in the middle area represents 1 byte as a 2 digit hexidecimal number. There are 6 lines, each with 32 pairs, so that is 192 bytes. The area at the right shows what the data looks like as ASCII text, if it looks like anything. "4d 41 50 53" corresponds to the "MAPS" text.