0000: 20        clrp
0001: cd cf     mov   x,#$cf
0003: bd        mov   sp,x
0004: e8 00     mov   a,#$00
0006: c5 86 03  mov   $0386,a
0009: c5 87 03  mov   $0387,a
000c: c5 88 03  mov   $0388,a
000f: c5 89 03  mov   $0389,a
0012: 5d        mov   x,a
0013: af        mov   (x)+,a
0014: c8 e8     cmp   x,#$e8
0016: d0 fb     bne   $0013
0018: e8 00     mov   a,#$00
001a: 5d        mov   x,a
001b: d5 00 02  mov   $0200+x,a
001e: 3d        inc   x
001f: d0 fa     bne   $001b
0021: d5 00 03  mov   $0300+x,a
0024: 3d        inc   x
0025: d0 fa     bne   $0021
0027: cd 0b     mov   x,#$0b
0029: f5 a1 12  mov   a,$12a1+x
002c: fd        mov   y,a
002d: f5 95 12  mov   a,$1295+x
0030: 3f 97 06  call  $0697
0033: 1d        dec   x
0034: 10 f3     bpl   $0029
0036: e8 f0     mov   a,#$f0
0038: c5 f1 00  mov   $00f1,a
003b: e8 10     mov   a,#$10
003d: c5 fa 00  mov   $00fa,a
0040: e8 36     mov   a,#$36
0042: c4 51     mov   $51,a
0044: e8 01     mov   a,#$01
0046: c5 f1 00  mov   $00f1,a
0049: ec fd 00  mov   y,$00fd
004c: f0 fb     beq   $0049
004e: 6d        push  y
004f: e8 38     mov   a,#$38
0051: cf        mul   ya
0052: 60        clrc
0053: 84 44     adc   a,$44
0055: c4 44     mov   $44,a
0057: 90 1a     bcc   $0073
0059: ab 45     inc   $45
005b: 3f ae 06  call  $06ae
005e: cd 00     mov   x,#$00
0060: 3f a5 05  call  $05a5
0063: 3f e5 09  call  $09e5
0066: cd 01     mov   x,#$01
0068: 3f a5 05  call  $05a5
006b: 3f 16 08  call  $0816
006e: cd 03     mov   x,#$03
0070: 3f a5 05  call  $05a5
0073: e4 51     mov   a,$51
0075: ee        pop   y
0076: cf        mul   ya
0077: 60        clrc
0078: 84 49     adc   a,$49
007a: c4 49     mov   $49,a
007c: 90 0f     bcc   $008d
007e: e5 88 03  mov   a,$0388
0081: d0 03     bne   $0086
0083: 3f c0 0b  call  $0bc0
0086: cd 02     mov   x,#$02
0088: 3f a5 05  call  $05a5
008b: 2f bc     bra   $0049
008d: e4 06     mov   a,$06
008f: f0 b8     beq   $0049
0091: cd 0e     mov   x,#$0e
0093: 8f 80 48  mov   $48,#$80
0096: f4 31     mov   a,$31+x
0098: f0 03     beq   $009d
009a: 3f 98 11  call  $1198
009d: 4b 48     lsr   $48
009f: 1d        dec   x
00a0: 1d        dec   x
00a1: 10 f3     bpl   $0096
00a3: 2f a4     bra   $0049
00a5: 7d        mov   a,x
00a6: fd        mov   y,a
00a7: f4 04     mov   a,$04+x
00a9: d5 f4 00  mov   $00f4+x,a
00ac: f5 f4 00  mov   a,$00f4+x
00af: 75 f4 00  cmp   a,$00f4+x
00b2: d0 f8     bne   $00ac
00b4: fd        mov   y,a
00b5: f4 08     mov   a,$08+x
00b7: db 08     mov   $08+x,y
00b9: de 08 05  cbne  $08+x,$00c1
00bc: 8d 00     mov   y,#$00
00be: db 00     mov   $00+x,y
00c0: 6f        ret
00c1: db 00     mov   $00+x,y
00c3: dd        mov   a,y
00c4: 6f        ret
00c5: 68 d0     cmp   a,#$d0
00c7: b0 05     bcs   $00ce
00c9: 68 c6     cmp   a,#$c6
00cb: 90 16     bcc   $00e3
00cd: 6f        ret
00ce: d4 c1     mov   $c1+x,a
00d0: 80        setc
00d1: a8 d0     sbc   a,#$d0
00d3: 8d 06     mov   y,#$06
00d5: 8f a5 14  mov   $14,#$a5
00d8: 8f 5f 15  mov   $15,#$5f
00db: 3f 56 0d  call  $0d56
00de: d0 ed     bne   $00cd
00e0: fc        inc   y
00e1: f7 14     mov   a,($14)+y
00e3: 28 7f     and   a,#$7f
00e5: 60        clrc
00e6: 84 43     adc   a,$43
00e8: d5 b1 02  mov   $02b1+x,a
00eb: f5 d1 02  mov   a,$02d1+x
00ee: d5 b0 02  mov   $02b0+x,a
00f1: e8 00     mov   a,#$00
00f3: d5 30 03  mov   $0330+x,a
00f6: d5 60 03  mov   $0360+x,a
00f9: d4 a0     mov   $a0+x,a
00fb: d5 10 01  mov   $0110+x,a
00fe: d4 b0     mov   $b0+x,a
0100: 09 48 5c  or    ($5c),($48)
0103: 09 48 47  or    ($47),($48)
0106: f5 00 03  mov   a,$0300+x
0109: d4 90     mov   $90+x,a
010b: f0 1e     beq   $012b
010d: f5 01 03  mov   a,$0301+x
0110: d4 91     mov   $91+x,a
0112: f5 20 03  mov   a,$0320+x
0115: d0 0a     bne   $0121
0117: f5 b1 02  mov   a,$02b1+x
011a: 80        setc
011b: b5 21 03  sbc   a,$0321+x
011e: d5 b1 02  mov   $02b1+x,a
0121: f5 21 03  mov   a,$0321+x
0124: 60        clrc
0125: 95 b1 02  adc   a,$02b1+x
0128: 3f 5d 0f  call  $0f5d
012b: f5 b1 02  mov   a,$02b1+x
012e: fd        mov   y,a
012f: f5 b0 02  mov   a,$02b0+x
0132: da 10     movw  $10,ya
0134: 8d 00     mov   y,#$00
0136: e4 11     mov   a,$11
0138: 80        setc
0139: a8 34     sbc   a,#$34
013b: b0 09     bcs   $0146
013d: e4 11     mov   a,$11
013f: 80        setc
0140: a8 13     sbc   a,#$13
0142: b0 06     bcs   $014a
0144: dc        dec   y
0145: 1c        asl   a
0146: 7a 10     addw  ya,$10
0148: da 10     movw  $10,ya
014a: 4d        push  x
014b: e4 11     mov   a,$11
014d: 3f bd 12  call  $12bd
0150: da 14     movw  $14,ya
0152: e4 11     mov   a,$11
0154: bc        inc   a
0155: 3f bd 12  call  $12bd
0158: ce        pop   x
0159: 9a 14     subw  ya,$14
015b: 2d        push  a
015c: e4 10     mov   a,$10
015e: cf        mul   ya
015f: 7a 14     addw  ya,$14
0161: da 14     movw  $14,ya
0163: e4 10     mov   a,$10
0165: ee        pop   y
0166: cf        mul   ya
0167: dd        mov   a,y
0168: 8d 00     mov   y,#$00
016a: 7a 14     addw  ya,$14
016c: da 14     movw  $14,ya
016e: f5 10 02  mov   a,$0210+x
0171: eb 14     mov   y,$14
0173: cf        mul   ya
0174: da 16     movw  $16,ya
0176: f5 10 02  mov   a,$0210+x
0179: eb 15     mov   y,$15
017b: cf        mul   ya
017c: 60        clrc
017d: 84 17     adc   a,$17
017f: c4 17     mov   $17,a
0181: 7d        mov   a,x
0182: 9f        xcn   a
0183: 5c        lsr   a
0184: 08 02     or    a,#$02
0186: fd        mov   y,a
0187: e4 16     mov   a,$16
0189: 3f 8f 06  call  $068f
018c: fc        inc   y
018d: e4 17     mov   a,$17
018f: 2d        push  a
0190: e4 48     mov   a,$48
0192: 24 1d     and   a,$1d
0194: ae        pop   a
0195: d0 06     bne   $019d
0197: cc f2 00  mov   $00f2,y
019a: c5 f3 00  mov   $00f3,a
019d: 6f        ret
019e: e8 0a     mov   a,#$0a
01a0: c5 87 03  mov   $0387,a
01a3: e4 51     mov   a,$51
01a5: 3f 14 0e  call  $0e14
01a8: e8 1d     mov   a,#$1d
01aa: c4 03     mov   $03,a
01ac: 2f 24     bra   $01d2
01ae: 78 12 00  cmp   $00,#$12
01b1: f0 0f     beq   $01c2
01b3: 78 11 00  cmp   $00,#$11
01b6: f0 0a     beq   $01c2
01b8: 78 11 04  cmp   $04,#$11
01bb: f0 0b     beq   $01c8
01bd: 78 1d 04  cmp   $04,#$1d
01c0: f0 06     beq   $01c8
01c2: e4 00     mov   a,$00
01c4: 30 d8     bmi   $019e
01c6: d0 0a     bne   $01d2
01c8: e5 82 03  mov   a,$0382
01cb: d0 4d     bne   $021a
01cd: e4 04     mov   a,$04
01cf: d0 7c     bne   $024d
01d1: 6f        ret
01d2: c4 04     mov   $04,a
01d4: e5 88 03  mov   a,$0388
01d7: f0 1e     beq   $01f7
01d9: e8 00     mov   a,#$00
01db: c5 88 03  mov   $0388,a
01de: e5 89 03  mov   a,$0389
01e1: d0 04     bne   $01e7
01e3: e8 20     mov   a,#$20
01e5: 2f 0b     bra   $01f2
01e7: e8 16     mov   a,#$16
01e9: c4 62     mov   $62,a
01eb: c4 64     mov   $64,a
01ed: 3f eb 0e  call  $0eeb
01f0: e8 00     mov   a,#$00
01f2: 8d 6c     mov   y,#$6c
01f4: 3f 97 06  call  $0697
01f7: e8 02     mov   a,#$02
01f9: c5 82 03  mov   $0382,a
01fc: 78 11 04  cmp   $04,#$11
01ff: d0 0a     bne   $020b
0201: e5 89 03  mov   a,$0389
0204: f0 05     beq   $020b
0206: e8 00     mov   a,#$00
0208: 3f 22 0f  call  $0f22
020b: e8 10     mov   a,#$10
020d: 8d 5c     mov   y,#$5c
020f: 3f 97 06  call  $0697
0212: 82 1d     set4  $1d
0214: e8 00     mov   a,#$00
0216: c5 08 03  mov   $0308,a
0219: 6f        ret
021a: 8c 82 03  dec   $0382
021d: d0 b2     bne   $01d1
021f: e4 04     mov   a,$04
0221: 1c        asl   a
0222: fd        mov   y,a
0223: f6 81 56  mov   a,$5681+y
0226: c4 18     mov   $18,a
0228: f6 82 56  mov   a,$5682+y
022b: c4 19     mov   $19,a
022d: 2f 25     bra   $0254
022f: 78 11 04  cmp   $04,#$11
0232: d0 0a     bne   $023e
0234: e8 60     mov   a,#$60
0236: c5 88 03  mov   $0388,a
0239: 8d 6c     mov   y,#$6c
023b: 3f 97 06  call  $0697
023e: 8f 00 04  mov   $04,#$00
0241: 92 1d     clr4  $1d
0243: cd 08     mov   x,#$08
0245: e4 c9     mov   a,$c9
0247: f0 03     beq   $024c
0249: 5f 4b 0d  jmp   $0d4b
024c: 6f        ret
024d: 8c 80 03  dec   $0380
0250: d0 54     bne   $02a6
0252: 3a 18     incw  $18
0254: cd 00     mov   x,#$00
0256: e7 18     mov   a,($18+x)
0258: f0 d5     beq   $022f
025a: 30 2a     bmi   $0286
025c: c5 81 03  mov   $0381,a
025f: 3a 18     incw  $18
0261: e7 18     mov   a,($18+x)
0263: c4 10     mov   $10,a
0265: 30 1f     bmi   $0286
0267: 8d 40     mov   y,#$40
0269: 3f 97 06  call  $0697
026c: 3a 18     incw  $18
026e: e7 18     mov   a,($18+x)
0270: 10 0b     bpl   $027d
0272: 5d        mov   x,a
0273: e4 10     mov   a,$10
0275: 8d 41     mov   y,#$41
0277: 3f 97 06  call  $0697
027a: 7d        mov   a,x
027b: 2f 09     bra   $0286
027d: 8d 41     mov   y,#$41
027f: 3f 97 06  call  $0697
0282: 3a 18     incw  $18
0284: e7 18     mov   a,($18+x)
0286: 68 da     cmp   a,#$da
0288: f0 69     beq   $02f3
028a: 68 dd     cmp   a,#$dd
028c: f0 34     beq   $02c2
028e: 68 eb     cmp   a,#$eb
0290: f0 43     beq   $02d5
0292: 68 ff     cmp   a,#$ff
0294: f0 89     beq   $021f
0296: cd 08     mov   x,#$08
0298: 3f c5 05  call  $05c5
029b: e8 10     mov   a,#$10
029d: 3f 32 0d  call  $0d32
02a0: e5 81 03  mov   a,$0381
02a3: c5 80 03  mov   $0380,a
02a6: f2 13     clr7  $13
02a8: cd 08     mov   x,#$08
02aa: f4 90     mov   a,$90+x
02ac: f0 05     beq   $02b3
02ae: 3f cd 09  call  $09cd
02b1: 2f 0e     bra   $02c1
02b3: e8 02     mov   a,#$02
02b5: 65 80 03  cmp   a,$0380
02b8: d0 07     bne   $02c1
02ba: e8 10     mov   a,#$10
02bc: 8d 5c     mov   y,#$5c
02be: 3f 97 06  call  $0697
02c1: 6f        ret
02c2: cd 00     mov   x,#$00
02c4: 3a 18     incw  $18
02c6: e7 18     mov   a,($18+x)
02c8: 8f 08 46  mov   $46,#$08
02cb: cd 08     mov   x,#$08
02cd: 3f c5 05  call  $05c5
02d0: e8 10     mov   a,#$10
02d2: 3f 32 0d  call  $0d32
02d5: cd 00     mov   x,#$00
02d7: 3a 18     incw  $18
02d9: e7 18     mov   a,($18+x)
02db: c4 99     mov   $99,a
02dd: 3a 18     incw  $18
02df: e7 18     mov   a,($18+x)
02e1: c4 98     mov   $98,a
02e3: 2d        push  a
02e4: 3a 18     incw  $18
02e6: e7 18     mov   a,($18+x)
02e8: ee        pop   y
02e9: 8f 08 46  mov   $46,#$08
02ec: cd 08     mov   x,#$08
02ee: 3f 5d 0f  call  $0f5d
02f1: 2f ad     bra   $02a0
02f3: cd 00     mov   x,#$00
02f5: 3a 18     incw  $18
02f7: e7 18     mov   a,($18+x)
02f9: 8d 09     mov   y,#$09
02fb: cf        mul   ya
02fc: 5d        mov   x,a
02fd: 8d 40     mov   y,#$40
02ff: 8f 08 12  mov   $12,#$08
0302: f5 70 55  mov   a,$5570+x
0305: 3f 97 06  call  $0697
0308: 3d        inc   x
0309: fc        inc   y
030a: 6e 12 f5  dbnz  $12,$0302
030d: f5 70 55  mov   a,$5570+x
0310: c5 18 02  mov   $0218,a
0313: 5f 52 07  jmp   $0752
0316: 78 24 07  cmp   $07,#$24
0319: f0 13     beq   $032e
031b: 78 24 03  cmp   $03,#$24
031e: f0 0a     beq   $032a
0320: 78 1d 07  cmp   $07,#$1d
0323: f0 09     beq   $032e
0325: 78 05 07  cmp   $07,#$05
0328: f0 04     beq   $032e
032a: e4 03     mov   a,$03
032c: d0 09     bne   $0337
032e: e4 0d     mov   a,$0d
0330: d0 19     bne   $034b
0332: e4 07     mov   a,$07
0334: d0 40     bne   $0376
0336: 6f        ret
0337: c4 07     mov   $07,a
0339: 8f 02 0d  mov   $0d,#$02
033c: e8 40     mov   a,#$40
033e: 8d 5c     mov   y,#$5c
0340: 3f 97 06  call  $0697
0343: c2 1d     set6  $1d
0345: e8 00     mov   a,#$00
0347: c5 0c 03  mov   $030c,a
034a: 6f        ret
034b: 6e 0d e8  dbnz  $0d,$0336
034e: e4 07     mov   a,$07
0350: 1c        asl   a
0351: fd        mov   y,a
0352: f6 19 56  mov   a,$5619+y
0355: c4 1a     mov   $1a,a
0357: f6 1a 56  mov   a,$561a+y
035a: c4 1b     mov   $1b,a
035c: 2f 1f     bra   $037d
035e: 8f 00 07  mov   $07,#$00
0361: d2 1d     clr6  $1d
0363: e8 00     mov   a,#$00
0365: c4 2f     mov   $2f,a
0367: 8d 3d     mov   y,#$3d
0369: 3f 97 06  call  $0697
036c: cd 0c     mov   x,#$0c
036e: e4 cd     mov   a,$cd
0370: f0 03     beq   $0375
0372: 5f 4b 0d  jmp   $0d4b
0375: 6f        ret
0376: 8c 84 03  dec   $0384
0379: d0 58     bne   $03d3
037b: 3a 1a     incw  $1a
037d: cd 00     mov   x,#$00
037f: e7 1a     mov   a,($1a+x)
0381: f0 db     beq   $035e
0383: 30 2a     bmi   $03af
0385: c5 85 03  mov   $0385,a
0388: 3a 1a     incw  $1a
038a: e7 1a     mov   a,($1a+x)
038c: c4 10     mov   $10,a
038e: 30 1f     bmi   $03af
0390: 8d 60     mov   y,#$60
0392: 3f 97 06  call  $0697
0395: 3a 1a     incw  $1a
0397: e7 1a     mov   a,($1a+x)
0399: 10 0b     bpl   $03a6
039b: 5d        mov   x,a
039c: e4 10     mov   a,$10
039e: 8d 61     mov   y,#$61
03a0: 3f 97 06  call  $0697
03a3: 7d        mov   a,x
03a4: 2f 09     bra   $03af
03a6: 8d 61     mov   y,#$61
03a8: 3f 97 06  call  $0697
03ab: 3a 1a     incw  $1a
03ad: e7 1a     mov   a,($1a+x)
03af: 68 da     cmp   a,#$da
03b1: f0 6d     beq   $0420
03b3: 68 dd     cmp   a,#$dd
03b5: f0 38     beq   $03ef
03b7: 68 eb     cmp   a,#$eb
03b9: f0 47     beq   $0402
03bb: 68 ff     cmp   a,#$ff
03bd: d0 04     bne   $03c3
03bf: 1a 1a     decw  $1a
03c1: 2f ba     bra   $037d
03c3: cd 0c     mov   x,#$0c
03c5: 3f c5 05  call  $05c5
03c8: e8 40     mov   a,#$40
03ca: 3f 32 0d  call  $0d32
03cd: e5 85 03  mov   a,$0385
03d0: c5 84 03  mov   $0384,a
03d3: f2 13     clr7  $13
03d5: cd 0c     mov   x,#$0c
03d7: f4 90     mov   a,$90+x
03d9: f0 05     beq   $03e0
03db: 3f cd 09  call  $09cd
03de: 2f 0e     bra   $03ee
03e0: e8 02     mov   a,#$02
03e2: 65 84 03  cmp   a,$0384
03e5: d0 07     bne   $03ee
03e7: e8 40     mov   a,#$40
03e9: 8d 5c     mov   y,#$5c
03eb: 3f 97 06  call  $0697
03ee: 6f        ret
03ef: cd 00     mov   x,#$00
03f1: 3a 1a     incw  $1a
03f3: e7 1a     mov   a,($1a+x)
03f5: 8f 0c 46  mov   $46,#$0c
03f8: cd 0c     mov   x,#$0c
03fa: 3f c5 05  call  $05c5
03fd: e8 40     mov   a,#$40
03ff: 3f 32 0d  call  $0d32
0402: cd 00     mov   x,#$00
0404: 3a 1a     incw  $1a
0406: e7 1a     mov   a,($1a+x)
0408: c4 9d     mov   $9d,a
040a: 3a 1a     incw  $1a
040c: e7 1a     mov   a,($1a+x)
040e: c4 9c     mov   $9c,a
0410: 2d        push  a
0411: 3a 1a     incw  $1a
0413: e7 1a     mov   a,($1a+x)
0415: ee        pop   y
0416: 8f 0c 46  mov   $46,#$0c
0419: cd 0c     mov   x,#$0c
041b: 3f 5d 0f  call  $0f5d
041e: 2f ad     bra   $03cd
0420: e8 00     mov   a,#$00
0422: c4 2f     mov   $2f,a
0424: 8d 3d     mov   y,#$3d
0426: 3f 97 06  call  $0697
0429: cd 00     mov   x,#$00
042b: 3a 1a     incw  $1a
042d: e7 1a     mov   a,($1a+x)
042f: 30 1d     bmi   $044e
0431: 8d 09     mov   y,#$09
0433: cf        mul   ya
0434: 5d        mov   x,a
0435: 8d 60     mov   y,#$60
0437: 8f 08 12  mov   $12,#$08
043a: f5 70 55  mov   a,$5570+x
043d: 3f 97 06  call  $0697
0440: 3d        inc   x
0441: fc        inc   y
0442: 6e 12 f5  dbnz  $12,$043a
0445: f5 70 55  mov   a,$5570+x
0448: c5 1c 02  mov   $021c,a
044b: 5f 7b 08  jmp   $087b
044e: 28 1f     and   a,#$1f
0450: c4 2e     mov   $2e,a
0452: 8d 6c     mov   y,#$6c
0454: 3f 97 06  call  $0697
0457: e8 40     mov   a,#$40
0459: c4 2f     mov   $2f,a
045b: 8d 3d     mov   y,#$3d
045d: 3f 97 06  call  $0697
0460: 2f c7     bra   $0429
0462: 8d 09     mov   y,#$09
0464: cf        mul   ya
0465: 5d        mov   x,a
0466: 8d 50     mov   y,#$50
0468: 8f 08 12  mov   $12,#$08
046b: f5 70 55  mov   a,$5570+x
046e: 3f 97 06  call  $0697
0471: 3d        inc   x
0472: fc        inc   y
0473: 6e 12 f5  dbnz  $12,$046b
0476: f5 70 55  mov   a,$5570+x
0479: c5 1a 02  mov   $021a,a
047c: 6f        ret
047d: e4 06     mov   a,$06
047f: 68 06     cmp   a,#$06
0481: f0 04     beq   $0487
0483: 28 fc     and   a,#$fc
0485: d0 7c     bne   $0503
0487: e5 86 03  mov   a,$0386
048a: d0 0e     bne   $049a
048c: e8 09     mov   a,#$09
048e: 3f 62 09  call  $0962
0491: e8 01     mov   a,#$01
0493: d0 02     bne   $0497
0495: e8 00     mov   a,#$00
0497: c5 86 03  mov   $0386,a
049a: 2f 67     bra   $0503
049c: e8 60     mov   a,#$60
049e: 8d 6c     mov   y,#$6c
04a0: 3f 97 06  call  $0697
04a3: e8 ff     mov   a,#$ff
04a5: 8d 5c     mov   y,#$5c
04a7: 3f 97 06  call  $0697
04aa: 3f f2 12  call  $12f2
04ad: e8 00     mov   a,#$00
04af: c4 04     mov   $04,a
04b1: c4 05     mov   $05,a
04b3: c4 06     mov   $06,a
04b5: c4 07     mov   $07,a
04b7: c4 1d     mov   $1d,a
04b9: c5 87 03  mov   $0387,a
04bc: c5 88 03  mov   $0388,a
04bf: c5 86 03  mov   $0386,a
04c2: c5 89 03  mov   $0389,a
04c5: e8 20     mov   a,#$20
04c7: 8d 6c     mov   y,#$6c
04c9: 3f 97 06  call  $0697
04cc: 6f        ret
04cd: e8 b0     mov   a,#$b0
04cf: 8d 02     mov   y,#$02
04d1: 9b 90     dec   $90+x
04d3: 3f 75 10  call  $1075
04d6: f5 b1 02  mov   a,$02b1+x
04d9: fd        mov   y,a
04da: f5 b0 02  mov   a,$02b0+x
04dd: da 10     movw  $10,ya
04df: 8f 00 48  mov   $48,#$00
04e2: 5f 34 06  jmp   $0634
04e5: e4 01     mov   a,$01
04e7: 68 ff     cmp   a,#$ff
04e9: f0 b1     beq   $049c
04eb: 68 02     cmp   a,#$02
04ed: f0 8e     beq   $047d
04ef: 68 03     cmp   a,#$03
04f1: f0 a2     beq   $0495
04f3: 68 01     cmp   a,#$01
04f5: f0 1d     beq   $0514
04f7: e4 05     mov   a,$05
04f9: 68 01     cmp   a,#$01
04fb: f0 06     beq   $0503
04fd: e4 01     mov   a,$01
04ff: 68 04     cmp   a,#$04
0501: f0 0b     beq   $050e
0503: e4 05     mov   a,$05
0505: 68 01     cmp   a,#$01
0507: f0 48     beq   $0551
0509: 68 04     cmp   a,#$04
050b: f0 04     beq   $0511
050d: 6f        ret
050e: 5f ce 0a  jmp   $0ace
0511: 5f 08 0b  jmp   $0b08
0514: c4 05     mov   $05,a
0516: e8 04     mov   a,#$04
0518: c5 83 03  mov   $0383,a
051b: e8 80     mov   a,#$80
051d: 8d 5c     mov   y,#$5c
051f: 3f 97 06  call  $0697
0522: e2 1d     set7  $1d
0524: e8 00     mov   a,#$00
0526: 8d 20     mov   y,#$20
0528: d6 ff 02  mov   $02ff+y,a
052b: fe fb     dbnz  y,$0528
052d: 6f        ret
052e: 8c 83 03  dec   $0383
0531: d0 da     bne   $050d
0533: 8f 30 1c  mov   $1c,#$30
0536: 2f 30     bra   $0568
0538: 78 2a 1c  cmp   $1c,#$2a
053b: d0 5c     bne   $0599
053d: 8f 0e 46  mov   $46,#$0e
0540: cd 0e     mov   x,#$0e
0542: 8d 00     mov   y,#$00
0544: cb 9f     mov   $9f,y
0546: 8d 12     mov   y,#$12
0548: cb 9e     mov   $9e,y
054a: e8 b9     mov   a,#$b9
054c: 3f 5d 0f  call  $0f5d
054f: 2f 48     bra   $0599
0551: e5 83 03  mov   a,$0383
0554: d0 d8     bne   $052e
0556: 6e 1c df  dbnz  $1c,$0538
0559: 8f 00 05  mov   $05,#$00
055c: f2 1d     clr7  $1d
055e: cd 0e     mov   x,#$0e
0560: e4 cf     mov   a,$cf
0562: f0 03     beq   $0567
0564: 5f 4b 0d  jmp   $0d4b
0567: 6f        ret
0568: 3f b1 0a  call  $0ab1
056b: e8 b2     mov   a,#$b2
056d: 8f 0e 46  mov   $46,#$0e
0570: cd 0e     mov   x,#$0e
0572: 3f c5 05  call  $05c5
0575: 8d 00     mov   y,#$00
0577: cb 9f     mov   $9f,y
0579: 8d 05     mov   y,#$05
057b: cb 9e     mov   $9e,y
057d: e8 b5     mov   a,#$b5
057f: 3f 5d 0f  call  $0f5d
0582: e8 38     mov   a,#$38
0584: c4 10     mov   $10,a
0586: 8d 70     mov   y,#$70
0588: 3f 97 06  call  $0697
058b: e8 38     mov   a,#$38
058d: c4 10     mov   $10,a
058f: 8d 71     mov   y,#$71
0591: 3f 97 06  call  $0697
0594: e8 80     mov   a,#$80
0596: 3f 32 0d  call  $0d32
0599: e8 02     mov   a,#$02
059b: 2e 1c 07  cbne  $1c,$05a5
059e: e8 80     mov   a,#$80
05a0: 8d 5c     mov   y,#$5c
05a2: 3f 97 06  call  $0697
05a5: f2 13     clr7  $13
05a7: e4 9e     mov   a,$9e
05a9: f0 05     beq   $05b0
05ab: cd 0e     mov   x,#$0e
05ad: 3f cd 09  call  $09cd
05b0: 6f        ret
05b1: e8 08     mov   a,#$08
05b3: 8d 09     mov   y,#$09
05b5: cf        mul   ya
05b6: 5d        mov   x,a
05b7: 8d 70     mov   y,#$70
05b9: 8f 08 12  mov   $12,#$08
05bc: f5 70 55  mov   a,$5570+x
05bf: 3f 97 06  call  $0697
05c2: 3d        inc   x
05c3: fc        inc   y
05c4: 6e 12 f5  dbnz  $12,$05bc
05c7: f5 70 55  mov   a,$5570+x
05ca: c5 1e 02  mov   $021e,a
05cd: 6f        ret
05ce: c4 05     mov   $05,a
05d0: e8 04     mov   a,#$04
05d2: c5 83 03  mov   $0383,a
05d5: e8 80     mov   a,#$80
05d7: 8d 5c     mov   y,#$5c
05d9: 3f 97 06  call  $0697
05dc: e2 1d     set7  $1d
05de: e8 00     mov   a,#$00
05e0: 8d 20     mov   y,#$20
05e2: d6 ff 02  mov   $02ff+y,a
05e5: fe fb     dbnz  y,$05e2
05e7: 6f        ret
05e8: 8c 83 03  dec   $0383
05eb: d0 fa     bne   $05e7
05ed: 8f 18 1c  mov   $1c,#$18
05f0: 2f 05     bra   $05f7
05f2: 78 0c 1c  cmp   $1c,#$0c
05f5: d0 3c     bne   $0633
05f7: e8 07     mov   a,#$07
05f9: 3f b3 0a  call  $0ab3
05fc: e8 a4     mov   a,#$a4
05fe: 8f 0e 46  mov   $46,#$0e
0601: cd 0e     mov   x,#$0e
0603: 3f c5 05  call  $05c5
0606: 2f 14     bra   $061c
0608: e5 83 03  mov   a,$0383
060b: d0 db     bne   $05e8
060d: 6e 1c e2  dbnz  $1c,$05f2
0610: 8f 00 05  mov   $05,#$00
0613: f2 1d     clr7  $1d
0615: cd 0e     mov   x,#$0e
0617: e4 cf     mov   a,$cf
0619: 5f 4b 0d  jmp   $0d4b
061c: e8 28     mov   a,#$28
061e: c4 10     mov   $10,a
0620: 8d 70     mov   y,#$70
0622: 3f 97 06  call  $0697
0625: e8 28     mov   a,#$28
0627: c4 10     mov   $10,a
0629: 8d 71     mov   y,#$71
062b: 3f 97 06  call  $0697
062e: e8 80     mov   a,#$80
0630: 3f 32 0d  call  $0d32
0633: e8 02     mov   a,#$02
0635: 2e 1c 07  cbne  $1c,$063f
0638: e8 80     mov   a,#$80
063a: 8d 5c     mov   y,#$5c
063c: 3f 97 06  call  $0697
063f: 6f        ret
0640: 80        setc
0641: 68 16     cmp   a,#$16
0643: f0 10     beq   $0655
0645: 68 10     cmp   a,#$10
0647: f0 0c     beq   $0655
0649: 68 0f     cmp   a,#$0f
064b: f0 08     beq   $0655
064d: 68 09     cmp   a,#$09
064f: 90 09     bcc   $065a
0651: 68 0d     cmp   a,#$0d
0653: b0 05     bcs   $065a
0655: 8d 00     mov   y,#$00
0657: cc 87 03  mov   $0387,y
065a: c4 06     mov   $06,a
065c: 8f 02 0c  mov   $0c,#$02
065f: 1c        asl   a
0660: fd        mov   y,a
0661: f6 5e 13  mov   a,$135e+y
0664: c4 40     mov   $40,a
0666: f6 5f 13  mov   a,$135f+y
0669: c4 41     mov   $41,a
066b: cd 0e     mov   x,#$0e
066d: e8 0a     mov   a,#$0a
066f: d5 81 02  mov   $0281+x,a
0672: e8 ff     mov   a,#$ff
0674: d5 41 02  mov   $0241+x,a
0677: e8 00     mov   a,#$00
0679: d5 d1 02  mov   $02d1+x,a
067c: d4 81     mov   $81+x,a
067e: d4 80     mov   $80+x,a
0680: d4 a1     mov   $a1+x,a
0682: d4 b1     mov   $b1+x,a
0684: d4 c0     mov   $c0+x,a
0686: d4 c1     mov   $c1+x,a
0688: 1d        dec   x
0689: 1d        dec   x
068a: 10 e1     bpl   $066d
068c: c4 58     mov   $58,a
068e: c4 60     mov   $60,a
0690: c4 52     mov   $52,a
0692: c4 43     mov   $43,a
0694: 8f c0 57  mov   $57,#$c0
0697: 8f 36 51  mov   $51,#$36
069a: 8d 20     mov   y,#$20
069c: d6 ff 02  mov   $02ff+y,a
069f: fe fb     dbnz  y,$069c
06a1: 2f 02     bra   $06a5
06a3: c4 06     mov   $06,a
06a5: e4 1d     mov   a,$1d
06a7: 48 ff     eor   a,#$ff
06a9: 8d 5c     mov   y,#$5c
06ab: 5f 97 06  jmp   $0697
06ae: cd f0     mov   x,#$f0
06b0: d8 58     mov   $58,x
06b2: e8 00     mov   a,#$00
06b4: c4 59     mov   $59,a
06b6: 80        setc
06b7: a4 57     sbc   a,$57
06b9: 3f 76 0f  call  $0f76
06bc: da 5a     movw  $5a,ya
06be: 2f 27     bra   $06e7
06c0: e4 06     mov   a,$06
06c2: f0 1a     beq   $06de
06c4: 68 06     cmp   a,#$06
06c6: f0 04     beq   $06cc
06c8: 28 fc     and   a,#$fc
06ca: d0 10     bne   $06dc
06cc: e5 86 03  mov   a,$0386
06cf: d0 0b     bne   $06dc
06d1: e8 20     mov   a,#$20
06d3: 8d 5c     mov   y,#$5c
06d5: 3f 97 06  call  $0697
06d8: a2 1d     set5  $1d
06da: 2f 02     bra   $06de
06dc: b2 1d     clr5  $1d
06de: e4 02     mov   a,$02
06e0: 30 cc     bmi   $06ae
06e2: f0 03     beq   $06e7
06e4: 5f 40 0b  jmp   $0b40
06e7: e4 0c     mov   a,$0c
06e9: d0 13     bne   $06fe
06eb: e4 06     mov   a,$06
06ed: d0 57     bne   $0746
06ef: 6f        ret
06f0: 8d 00     mov   y,#$00
06f2: f7 40     mov   a,($40)+y
06f4: 3a 40     incw  $40
06f6: 2d        push  a
06f7: f7 40     mov   a,($40)+y
06f9: 3a 40     incw  $40
06fb: fd        mov   y,a
06fc: ae        pop   a
06fd: 6f        ret
06fe: 6e 0c ee  dbnz  $0c,$06ef
0701: 3f f0 0b  call  $0bf0
0704: da 16     movw  $16,ya
0706: dd        mov   a,y
0707: d0 19     bne   $0722
0709: e4 16     mov   a,$16
070b: f0 96     beq   $06a3
070d: 8b 42     dec   $42
070f: f0 0b     beq   $071c
0711: 10 02     bpl   $0715
0713: c4 42     mov   $42,a
0715: 3f f0 0b  call  $0bf0
0718: da 40     movw  $40,ya
071a: 2f e5     bra   $0701
071c: 3a 40     incw  $40
071e: 3a 40     incw  $40
0720: 2f df     bra   $0701
0722: 8d 0f     mov   y,#$0f
0724: f7 16     mov   a,($16)+y
0726: d6 30 00  mov   $0030+y,a
0729: dc        dec   y
072a: 10 f8     bpl   $0724
072c: cd 0e     mov   x,#$0e
072e: 8f 80 48  mov   $48,#$80
0731: f4 31     mov   a,$31+x
0733: f0 0b     beq   $0740
0735: e8 01     mov   a,#$01
0737: d4 70     mov   $70+x,a
0739: f4 c1     mov   a,$c1+x
073b: d0 03     bne   $0740
073d: 3f 4a 0d  call  $0d4a
0740: 4b 48     lsr   $48
0742: 1d        dec   x
0743: 1d        dec   x
0744: 10 eb     bpl   $0731
0746: cd 00     mov   x,#$00
0748: d8 47     mov   $47,x
074a: 8f 01 48  mov   $48,#$01
074d: d8 46     mov   $46,x
074f: f4 31     mov   a,$31+x
0751: f0 76     beq   $07c9
0753: 9b 70     dec   $70+x
0755: d0 6f     bne   $07c6
0757: 3f 5e 12  call  $125e
075a: d0 1e     bne   $077a
075c: f4 c0     mov   a,$c0+x
075e: f0 a1     beq   $0701
0760: 9b c0     dec   $c0+x
0762: d0 0a     bne   $076e
0764: f5 e0 03  mov   a,$03e0+x
0767: d4 30     mov   $30+x,a
0769: f5 e1 03  mov   a,$03e1+x
076c: 2f 08     bra   $0776
076e: f5 f0 03  mov   a,$03f0+x
0771: d4 30     mov   $30+x,a
0773: f5 f1 03  mov   a,$03f1+x
0776: d4 31     mov   $31+x,a
0778: 2f dd     bra   $0757
077a: 30 23     bmi   $079f
077c: d5 00 02  mov   $0200+x,a
077f: 3f 5e 12  call  $125e
0782: 30 1b     bmi   $079f
0784: 2d        push  a
0785: 9f        xcn   a
0786: 28 07     and   a,#$07
0788: fd        mov   y,a
0789: f6 68 12  mov   a,$1268+y
078c: d5 01 02  mov   $0201+x,a
078f: ae        pop   a
0790: 28 0f     and   a,#$0f
0792: fd        mov   y,a
0793: f6 70 12  mov   a,$1270+y
0796: d5 11 02  mov   $0211+x,a
0799: 09 48 5c  or    ($5c),($48)
079c: 3f 5e 12  call  $125e
079f: 68 da     cmp   a,#$da
07a1: 90 05     bcc   $07a8
07a3: 3f 40 0d  call  $0d40
07a6: 2f af     bra   $0757
07a8: 2d        push  a
07a9: e4 48     mov   a,$48
07ab: 24 1d     and   a,$1d
07ad: ae        pop   a
07ae: d0 03     bne   $07b3
07b0: 3f c5 05  call  $05c5
07b3: f5 00 02  mov   a,$0200+x
07b6: d4 70     mov   $70+x,a
07b8: fd        mov   y,a
07b9: f5 01 02  mov   a,$0201+x
07bc: cf        mul   ya
07bd: dd        mov   a,y
07be: d0 01     bne   $07c1
07c0: bc        inc   a
07c1: d5 00 01  mov   $0100+x,a
07c4: 2f 03     bra   $07c9
07c6: 3f a1 10  call  $10a1
07c9: 3d        inc   x
07ca: 3d        inc   x
07cb: 0b 48     asl   $48
07cd: b0 03     bcs   $07d2
07cf: 5f 4d 0c  jmp   $0c4d
07d2: e4 52     mov   a,$52
07d4: f0 0d     beq   $07e3
07d6: 6e 52 04  dbnz  $52,$07dd
07d9: ba 52     movw  ya,$52
07db: 2f 04     bra   $07e1
07dd: ba 54     movw  ya,$54
07df: 7a 50     addw  ya,$50
07e1: da 50     movw  $50,ya
07e3: e4 60     mov   a,$60
07e5: f0 1c     beq   $0803
07e7: 6e 60 0a  dbnz  $60,$07f4
07ea: e8 00     mov   a,#$00
07ec: eb 62     mov   y,$62
07ee: da 61     movw  $61,ya
07f0: eb 64     mov   y,$64
07f2: 2f 0a     bra   $07fe
07f4: ba 65     movw  ya,$65
07f6: 7a 61     addw  ya,$61
07f8: da 61     movw  $61,ya
07fa: ba 67     movw  ya,$67
07fc: 7a 63     addw  ya,$63
07fe: da 63     movw  $63,ya
0800: 3f eb 0e  call  $0eeb
0803: e4 58     mov   a,$58
0805: f0 10     beq   $0817
0807: 6e 58 04  dbnz  $58,$080e
080a: ba 58     movw  ya,$58
080c: 2f 04     bra   $0812
080e: ba 5a     movw  ya,$5a
0810: 7a 56     addw  ya,$56
0812: da 56     movw  $56,ya
0814: 8f ff 5c  mov   $5c,#$ff
0817: cd 0e     mov   x,#$0e
0819: 8f 80 48  mov   $48,#$80
081c: f4 31     mov   a,$31+x
081e: f0 03     beq   $0823
0820: 3f db 0f  call  $0fdb
0823: 4b 48     lsr   $48
0825: 1d        dec   x
0826: 1d        dec   x
0827: 10 f3     bpl   $081c
0829: 8f 00 5c  mov   $5c,#$00
082c: e4 1d     mov   a,$1d
082e: 48 ff     eor   a,#$ff
0830: 24 47     and   a,$47
0832: 2d        push  a
0833: 8d 5c     mov   y,#$5c
0835: e8 00     mov   a,#$00
0837: 3f 97 06  call  $0697
083a: ae        pop   a
083b: 8d 4c     mov   y,#$4c
083d: 5f 97 06  jmp   $0697
0840: 1c        asl   a
0841: 5d        mov   x,a
0842: e8 00     mov   a,#$00
0844: 1f dc 0e  jmp   ($0edc+x)
0847: 3f 5c 12  call  $125c
084a: bc        inc   a
084b: d4 c1     mov   $c1+x,a
084d: 9c        dec   a
084e: 8d 05     mov   y,#$05
0850: 8f 46 14  mov   $14,#$46
0853: 8f 5f 15  mov   $15,#$5f
0856: cf        mul   ya
0857: 7a 14     addw  ya,$14
0859: da 14     movw  $14,ya
085b: e4 48     mov   a,$48
085d: 24 1d     and   a,$1d
085f: d0 2c     bne   $088d
0861: 4d        push  x
0862: 7d        mov   a,x
0863: 9f        xcn   a
0864: 5c        lsr   a
0865: 08 04     or    a,#$04
0867: 5d        mov   x,a
0868: e4 48     mov   a,$48
086a: 48 ff     eor   a,#$ff
086c: 24 2f     and   a,$2f
086e: c4 2f     mov   $2f,a
0870: 8d 3d     mov   y,#$3d
0872: 3f 97 06  call  $0697
0875: 8d 00     mov   y,#$00
0877: f7 14     mov   a,($14)+y
0879: c9 f2 00  mov   $00f2,x
087c: c5 f3 00  mov   $00f3,a
087f: 3d        inc   x
0880: fc        inc   y
0881: ad 04     cmp   y,#$04
0883: d0 f2     bne   $0877
0885: f7 14     mov   a,($14)+y
0887: ce        pop   x
0888: d5 10 02  mov   $0210+x,a
088b: e8 00     mov   a,#$00
088d: 6f        ret
088e: 3f 5c 12  call  $125c
0891: 28 1f     and   a,#$1f
0893: d5 81 02  mov   $0281+x,a
0896: dd        mov   a,y
0897: 28 c0     and   a,#$c0
0899: d5 a1 02  mov   $02a1+x,a
089c: e8 00     mov   a,#$00
089e: d5 80 02  mov   $0280+x,a
08a1: 09 48 5c  or    ($5c),($48)
08a4: 6f        ret
08a5: 3f 5c 12  call  $125c
08a8: d4 81     mov   $81+x,a
08aa: 2d        push  a
08ab: 3f 5e 12  call  $125e
08ae: d5 a0 02  mov   $02a0+x,a
08b1: 80        setc
08b2: b5 81 02  sbc   a,$0281+x
08b5: ce        pop   x
08b6: 3f 76 0f  call  $0f76
08b9: d5 90 02  mov   $0290+x,a
08bc: dd        mov   a,y
08bd: d5 91 02  mov   $0291+x,a
08c0: 6f        ret
08c1: 3f 5c 12  call  $125c
08c4: d5 40 03  mov   $0340+x,a
08c7: e8 00     mov   a,#$00
08c9: d5 41 03  mov   $0341+x,a
08cc: 3f 5e 12  call  $125e
08cf: d5 31 03  mov   $0331+x,a
08d2: 3f 5e 12  call  $125e
08d5: f8 46     mov   x,$46
08d7: d4 a1     mov   $a1+x,a
08d9: 6f        ret
08da: 3f 5c 12  call  $125c
08dd: d5 41 03  mov   $0341+x,a
08e0: 2d        push  a
08e1: f4 a1     mov   a,$a1+x
08e3: d5 51 03  mov   $0351+x,a
08e6: ce        pop   x
08e7: 8d 00     mov   y,#$00
08e9: 9e        div   ya,x
08ea: f8 46     mov   x,$46
08ec: d5 50 03  mov   $0350+x,a
08ef: 6f        ret
08f0: 3f 5c 12  call  $125c
08f3: c4 57     mov   $57,a
08f5: 8f 00 56  mov   $56,#$00
08f8: 8f ff 5c  mov   $5c,#$ff
08fb: 6f        ret
08fc: 3f 5c 12  call  $125c
08ff: c4 58     mov   $58,a
0901: 3f 5e 12  call  $125e
0904: c4 59     mov   $59,a
0906: f8 58     mov   x,$58
0908: 80        setc
0909: a4 57     sbc   a,$57
090b: 3f 76 0f  call  $0f76
090e: da 5a     movw  $5a,ya
0910: 6f        ret
0911: 3f 5c 12  call  $125c
0914: 85 87 03  adc   a,$0387
0917: c4 51     mov   $51,a
0919: 8f 00 50  mov   $50,#$00
091c: 6f        ret
091d: 3f 5c 12  call  $125c
0920: c4 52     mov   $52,a
0922: 3f 5e 12  call  $125e
0925: 85 87 03  adc   a,$0387
0928: c4 53     mov   $53,a
092a: f8 52     mov   x,$52
092c: 80        setc
092d: a4 51     sbc   a,$51
092f: 3f 76 0f  call  $0f76
0932: da 54     movw  $54,ya
0934: 6f        ret
0935: 3f 5c 12  call  $125c
0938: c4 43     mov   $43,a
093a: 6f        ret
093b: 3f 5c 12  call  $125c
093e: d5 70 03  mov   $0370+x,a
0941: 3f 5e 12  call  $125e
0944: d5 62 03  mov   $0362+x,a
0947: 3f 5e 12  call  $125e
094a: f8 46     mov   x,$46
094c: d4 b1     mov   $b1+x,a
094e: 6f        ret
094f: e8 01     mov   a,#$01
0951: 2f 02     bra   $0955
0953: e8 00     mov   a,#$00
0955: f8 46     mov   x,$46
0957: d5 20 03  mov   $0320+x,a
095a: 3f 5c 12  call  $125c
095d: d5 01 03  mov   $0301+x,a
0960: 3f 5e 12  call  $125e
0963: d5 00 03  mov   $0300+x,a
0966: 3f 5e 12  call  $125e
0969: d5 21 03  mov   $0321+x,a
096c: 6f        ret
096d: f8 46     mov   x,$46
096f: d5 00 03  mov   $0300+x,a
0972: 6f        ret
0973: 3f 5c 12  call  $125c
0976: d5 41 02  mov   $0241+x,a
0979: e8 00     mov   a,#$00
097b: d5 40 02  mov   $0240+x,a
097e: 09 48 5c  or    ($5c),($48)
0981: 6f        ret
0982: 3f 5c 12  call  $125c
0985: d4 80     mov   $80+x,a
0987: 2d        push  a
0988: 3f 5e 12  call  $125e
098b: d5 60 02  mov   $0260+x,a
098e: 80        setc
098f: b5 41 02  sbc   a,$0241+x
0992: ce        pop   x
0993: 3f 76 0f  call  $0f76
0996: d5 50 02  mov   $0250+x,a
0999: dd        mov   a,y
099a: d5 51 02  mov   $0251+x,a
099d: 6f        ret
099e: 3f 5c 12  call  $125c
09a1: d5 d1 02  mov   $02d1+x,a
09a4: 6f        ret
09a5: 3f 5c 12  call  $125c
09a8: 2d        push  a
09a9: 3f 5e 12  call  $125e
09ac: 2d        push  a
09ad: 3f 5e 12  call  $125e
09b0: d4 c0     mov   $c0+x,a
09b2: f4 30     mov   a,$30+x
09b4: d5 e0 03  mov   $03e0+x,a
09b7: f4 31     mov   a,$31+x
09b9: d5 e1 03  mov   $03e1+x,a
09bc: ae        pop   a
09bd: d4 31     mov   $31+x,a
09bf: d5 f1 03  mov   $03f1+x,a
09c2: ae        pop   a
09c3: d4 30     mov   $30+x,a
09c5: d5 f0 03  mov   $03f0+x,a
09c8: 6f        ret
09c9: 3f 5c 12  call  $125c
09cc: c5 89 03  mov   $0389,a
09cf: 8d 4d     mov   y,#$4d
09d1: 3f 97 06  call  $0697
09d4: 3f 5e 12  call  $125e
09d7: e8 00     mov   a,#$00
09d9: da 61     movw  $61,ya
09db: 3f 5e 12  call  $125e
09de: e8 00     mov   a,#$00
09e0: da 63     movw  $63,ya
09e2: c4 2e     mov   $2e,a
09e4: 28 1f     and   a,#$1f
09e6: 8d 6c     mov   y,#$6c
09e8: 3f 97 06  call  $0697
09eb: e4 62     mov   a,$62
09ed: 8d 2c     mov   y,#$2c
09ef: 3f 97 06  call  $0697
09f2: e4 64     mov   a,$64
09f4: 8d 3c     mov   y,#$3c
09f6: 5f 97 06  jmp   $0697
09f9: 3f 5c 12  call  $125c
09fc: c4 60     mov   $60,a
09fe: 3f 5e 12  call  $125e
0a01: c4 69     mov   $69,a
0a03: f8 60     mov   x,$60
0a05: 80        setc
0a06: a4 62     sbc   a,$62
0a08: 3f 76 0f  call  $0f76
0a0b: da 65     movw  $65,ya
0a0d: 3f 5e 12  call  $125e
0a10: c4 6a     mov   $6a,a
0a12: f8 60     mov   x,$60
0a14: 80        setc
0a15: a4 64     sbc   a,$64
0a17: 3f 76 0f  call  $0f76
0a1a: da 67     movw  $67,ya
0a1c: 6f        ret
0a1d: f8 46     mov   x,$46
0a1f: c5 89 03  mov   $0389,a
0a22: fd        mov   y,a
0a23: da 61     movw  $61,ya
0a25: da 63     movw  $63,ya
0a27: 3f eb 0e  call  $0eeb
0a2a: c4 2e     mov   $2e,a
0a2c: 08 20     or    a,#$20
0a2e: 8d 6c     mov   y,#$6c
0a30: 5f 97 06  jmp   $0697
0a33: 3f 5c 12  call  $125c
0a36: 8d 7d     mov   y,#$7d
0a38: 3f 97 06  call  $0697
0a3b: 3f 5e 12  call  $125e
0a3e: 8d 0d     mov   y,#$0d
0a40: 3f 97 06  call  $0697
0a43: 3f 5e 12  call  $125e
0a46: 8d 08     mov   y,#$08
0a48: cf        mul   ya
0a49: 5d        mov   x,a
0a4a: 8d 0f     mov   y,#$0f
0a4c: f5 ad 12  mov   a,$12ad+x
0a4f: 3f 97 06  call  $0697
0a52: 3d        inc   x
0a53: dd        mov   a,y
0a54: 60        clrc
0a55: 88 10     adc   a,#$10
0a57: fd        mov   y,a
0a58: 10 f2     bpl   $0a4c
0a5a: f8 46     mov   x,$46
0a5c: 6f        ret
0a5d: 28 7f     and   a,#$7f
0a5f: d5 d0 02  mov   $02d0+x,a
0a62: 80        setc
0a63: b5 b1 02  sbc   a,$02b1+x
0a66: 2d        push  a
0a67: f4 90     mov   a,$90+x
0a69: 5d        mov   x,a
0a6a: ae        pop   a
0a6b: 3f 76 0f  call  $0f76
0a6e: d5 c0 02  mov   $02c0+x,a
0a71: dd        mov   a,y
0a72: d5 c1 02  mov   $02c1+x,a
0a75: 6f        ret
0a76: b0 0d     bcs   $0a85
0a78: 48 ff     eor   a,#$ff
0a7a: bc        inc   a
0a7b: 3f 85 0f  call  $0f85
0a7e: da 14     movw  $14,ya
0a80: ba 0e     movw  ya,$0e
0a82: 9a 14     subw  ya,$14
0a84: 6f        ret
0a85: 8d 00     mov   y,#$00
0a87: 9e        div   ya,x
0a88: 2d        push  a
0a89: e8 00     mov   a,#$00
0a8b: 9e        div   ya,x
0a8c: ee        pop   y
0a8d: f8 46     mov   x,$46
0a8f: 6f        ret
0a90: 47 0d     eor   a,($0d+x)
0a92: 8e        pop   psw
0a93: 0d        push  psw
0a94: a5 0d 00  sbc   a,$000d
0a97: 00        nop
0a98: c1        tcall 12
0a99: 0d        push  psw
0a9a: d5 0d f0  mov   $f00d+x,a
0a9d: 0d        push  psw
0a9e: fc        inc   y
0a9f: 0d        push  psw
0aa0: 11        tcall 1
0aa1: 0e 1d 0e  tset1 $0e1d
0aa4: 35 0e 3b  and   a,$3b0e+x
0aa7: 0e 4a 0e  tset1 $0e4a
0aaa: 73 0e 82  bbc3  $0e,$0a2f
0aad: 0e a5 0e  tset1 $0ea5
0ab0: da 0d     movw  $0d,ya
0ab2: 4f 0e     pcall $0e
0ab4: 53 0e 00  bbc2  $0e,$0ab7
0ab7: 00        nop
0ab8: 9e        div   ya,x
0ab9: 0e c9 0e  tset1 $0ec9
0abc: 1d        dec   x
0abd: 0f        brk
0abe: 33 0f f9  bbc1  $0f,$0aba
0ac1: 0e 02 02  tset1 $0202
0ac4: 03 04 04  bbs0  $04,$0acb
0ac7: 01        tcall 0
0ac8: 02 03     set0  $03
0aca: 02 03     set0  $03
0acc: 02 04     set0  $04
0ace: 01        tcall 0
0acf: 02 03     set0  $03
0ad1: 04 02     or    a,$02
0ad3: 04 04     or    a,$04
0ad5: 01        tcall 0
0ad6: 02 04     set0  $04
0ad8: 01        tcall 0
0ad9: 04 04     or    a,$04
0adb: f4 80     mov   a,$80+x
0add: f0 0c     beq   $0aeb
0adf: 09 48 5c  or    ($5c),($48)
0ae2: e8 40     mov   a,#$40
0ae4: 8d 02     mov   y,#$02
0ae6: 9b 80     dec   $80+x
0ae8: 3f 75 10  call  $1075
0aeb: f4 b1     mov   a,$b1+x
0aed: fd        mov   y,a
0aee: f0 23     beq   $0b13
0af0: f5 70 03  mov   a,$0370+x
0af3: de b0 1b  cbne  $b0+x,$0b11
0af6: 09 48 5c  or    ($5c),($48)
0af9: f5 60 03  mov   a,$0360+x
0afc: 10 07     bpl   $0b05
0afe: fc        inc   y
0aff: d0 04     bne   $0b05
0b01: e8 80     mov   a,#$80
0b03: 2f 04     bra   $0b09
0b05: 60        clrc
0b06: 95 62 03  adc   a,$0362+x
0b09: d5 60 03  mov   $0360+x,a
0b0c: 3f 3a 12  call  $123a
0b0f: 2f 08     bra   $0b19
0b11: bb b0     inc   $b0+x
0b13: f5 11 02  mov   a,$0211+x
0b16: 3f 4d 12  call  $124d
0b19: f4 81     mov   a,$81+x
0b1b: d0 07     bne   $0b24
0b1d: e4 48     mov   a,$48
0b1f: 24 5c     and   a,$5c
0b21: d0 0a     bne   $0b2d
0b23: 6f        ret
0b24: e8 80     mov   a,#$80
0b26: 8d 02     mov   y,#$02
0b28: 9b 81     dec   $81+x
0b2a: 3f 75 10  call  $1075
0b2d: f5 81 02  mov   a,$0281+x
0b30: fd        mov   y,a
0b31: f5 80 02  mov   a,$0280+x
0b34: da 10     movw  $10,ya
0b36: 7d        mov   a,x
0b37: 9f        xcn   a
0b38: 5c        lsr   a
0b39: c4 12     mov   $12,a
0b3b: eb 11     mov   y,$11
0b3d: f6 81 12  mov   a,$1281+y
0b40: 80        setc
0b41: b6 80 12  sbc   a,$1280+y
0b44: eb 10     mov   y,$10
0b46: cf        mul   ya
0b47: dd        mov   a,y
0b48: eb 11     mov   y,$11
0b4a: 60        clrc
0b4b: 96 80 12  adc   a,$1280+y
0b4e: fd        mov   y,a
0b4f: f5 71 03  mov   a,$0371+x
0b52: cf        mul   ya
0b53: f5 a1 02  mov   a,$02a1+x
0b56: 13 12 01  bbc0  $12,$0b5a
0b59: 1c        asl   a
0b5a: 10 05     bpl   $0b61
0b5c: dd        mov   a,y
0b5d: 48 ff     eor   a,#$ff
0b5f: bc        inc   a
0b60: fd        mov   y,a
0b61: dd        mov   a,y
0b62: eb 12     mov   y,$12
0b64: 3f 8f 06  call  $068f
0b67: e8 00     mov   a,#$00
0b69: 8d 14     mov   y,#$14
0b6b: 9a 10     subw  ya,$10
0b6d: da 10     movw  $10,ya
0b6f: ab 12     inc   $12
0b71: 33 12 c7  bbc1  $12,$0b3b
0b74: 6f        ret
0b75: da 14     movw  $14,ya
0b77: d0 0f     bne   $0b88
0b79: 60        clrc
0b7a: 88 20     adc   a,#$20
0b7c: da 16     movw  $16,ya
0b7e: 7d        mov   a,x
0b7f: fd        mov   y,a
0b80: e8 00     mov   a,#$00
0b82: 2d        push  a
0b83: f7 16     mov   a,($16)+y
0b85: fc        inc   y
0b86: 2f 12     bra   $0b9a
0b88: 60        clrc
0b89: 88 10     adc   a,#$10
0b8b: da 16     movw  $16,ya
0b8d: 7d        mov   a,x
0b8e: fd        mov   y,a
0b8f: f7 14     mov   a,($14)+y
0b91: 60        clrc
0b92: 97 16     adc   a,($16)+y
0b94: 2d        push  a
0b95: fc        inc   y
0b96: f7 14     mov   a,($14)+y
0b98: 97 16     adc   a,($16)+y
0b9a: d7 14     mov   ($14)+y,a
0b9c: dc        dec   y
0b9d: ae        pop   a
0b9e: d7 14     mov   ($14)+y,a
0ba0: 6f        ret
0ba1: 40        setp
0ba2: 9b 00     dec   $00+x
0ba4: 20        clrp
0ba5: f0 05     beq   $0bac
0ba7: e8 02     mov   a,#$02
0ba9: de 70 2c  cbne  $70+x,$0bd8
0bac: f4 30     mov   a,$30+x
0bae: fb 31     mov   y,$31+x
0bb0: da 14     movw  $14,ya
0bb2: 8d 00     mov   y,#$00
0bb4: f7 14     mov   a,($14)+y
0bb6: f0 19     beq   $0bd1
0bb8: 30 05     bmi   $0bbf
0bba: fc        inc   y
0bbb: f7 14     mov   a,($14)+y
0bbd: 10 fb     bpl   $0bba
0bbf: 68 c6     cmp   a,#$c6
0bc1: f0 15     beq   $0bd8
0bc3: 68 da     cmp   a,#$da
0bc5: 90 0a     bcc   $0bd1
0bc7: 6d        push  y
0bc8: fd        mov   y,a
0bc9: ae        pop   a
0bca: 60        clrc
0bcb: 96 e8 0e  adc   a,$0ee8+y
0bce: fd        mov   y,a
0bcf: 2f e3     bra   $0bb4
0bd1: e4 48     mov   a,$48
0bd3: 8d 5c     mov   y,#$5c
0bd5: 3f 8f 06  call  $068f
0bd8: f2 13     clr7  $13
0bda: f4 90     mov   a,$90+x
0bdc: f0 06     beq   $0be4
0bde: e4 48     mov   a,$48
0be0: 24 1d     and   a,$1d
0be2: f0 2d     beq   $0c11
0be4: e7 30     mov   a,($30+x)
0be6: 68 dd     cmp   a,#$dd
0be8: d0 40     bne   $0c2a
0bea: e4 48     mov   a,$48
0bec: 24 1d     and   a,$1d
0bee: f0 0b     beq   $0bfb
0bf0: 8f 04 10  mov   $10,#$04
0bf3: 3f 60 12  call  $1260
0bf6: 6e 10 fa  dbnz  $10,$0bf3
0bf9: 2f 16     bra   $0c11
0bfb: 3f 60 12  call  $1260
0bfe: 3f 5e 12  call  $125e
0c01: d4 91     mov   $91+x,a
0c03: 3f 5e 12  call  $125e
0c06: d4 90     mov   $90+x,a
0c08: 3f 5e 12  call  $125e
0c0b: 60        clrc
0c0c: 84 43     adc   a,$43
0c0e: 3f 5d 0f  call  $0f5d
0c11: f4 91     mov   a,$91+x
0c13: f0 04     beq   $0c19
0c15: 9b 91     dec   $91+x
0c17: 2f 11     bra   $0c2a
0c19: e4 1d     mov   a,$1d
0c1b: 24 48     and   a,$48
0c1d: d0 0b     bne   $0c2a
0c1f: e2 13     set7  $13
0c21: e8 b0     mov   a,#$b0
0c23: 8d 02     mov   y,#$02
0c25: 9b 90     dec   $90+x
0c27: 3f 75 10  call  $1075
0c2a: f5 b1 02  mov   a,$02b1+x
0c2d: fd        mov   y,a
0c2e: f5 b0 02  mov   a,$02b0+x
0c31: da 10     movw  $10,ya
0c33: f4 a1     mov   a,$a1+x
0c35: f0 09     beq   $0c40
0c37: f5 40 03  mov   a,$0340+x
0c3a: 74 a0     cmp   a,$a0+x
0c3c: f0 06     beq   $0c44
0c3e: bb a0     inc   $a0+x
0c40: e3 13 52  bbs7  $13,$0c95
0c43: 6f        ret
0c44: f5 41 03  mov   a,$0341+x
0c47: f0 1d     beq   $0c66
0c49: 75 10 01  cmp   a,$0110+x
0c4c: d0 07     bne   $0c55
0c4e: f5 51 03  mov   a,$0351+x
0c51: d4 a1     mov   $a1+x,a
0c53: 2f 11     bra   $0c66
0c55: f5 10 01  mov   a,$0110+x
0c58: f0 02     beq   $0c5c
0c5a: f4 a1     mov   a,$a1+x
0c5c: 60        clrc
0c5d: 95 50 03  adc   a,$0350+x
0c60: d4 a1     mov   $a1+x,a
0c62: 40        setp
0c63: bb 10     inc   $10+x
0c65: 20        clrp
0c66: f5 30 03  mov   a,$0330+x
0c69: 60        clrc
0c6a: 95 31 03  adc   a,$0331+x
0c6d: d5 30 03  mov   $0330+x,a
0c70: c4 12     mov   $12,a
0c72: 1c        asl   a
0c73: 1c        asl   a
0c74: 90 02     bcc   $0c78
0c76: 48 ff     eor   a,#$ff
0c78: fd        mov   y,a
0c79: f4 a1     mov   a,$a1+x
0c7b: 68 f1     cmp   a,#$f1
0c7d: b0 06     bcs   $0c85
0c7f: cf        mul   ya
0c80: dd        mov   a,y
0c81: 8d 00     mov   y,#$00
0c83: 2f 03     bra   $0c88
0c85: 28 0f     and   a,#$0f
0c87: cf        mul   ya
0c88: f3 12 06  bbc7  $12,$0c91
0c8b: da 12     movw  $12,ya
0c8d: ba 0e     movw  ya,$0e
0c8f: 9a 12     subw  ya,$12
0c91: 7a 10     addw  ya,$10
0c93: da 10     movw  $10,ya
0c95: 5f 34 06  jmp   $0634
0c98: f2 13     clr7  $13
0c9a: f4 b1     mov   a,$b1+x
0c9c: f0 09     beq   $0ca7
0c9e: f5 70 03  mov   a,$0370+x
0ca1: de b0 03  cbne  $b0+x,$0ca7
0ca4: 3f 2d 12  call  $122d
0ca7: f5 81 02  mov   a,$0281+x
0caa: fd        mov   y,a
0cab: f5 80 02  mov   a,$0280+x
0cae: da 10     movw  $10,ya
0cb0: f4 81     mov   a,$81+x
0cb2: d0 05     bne   $0cb9
0cb4: e3 13 0c  bbs7  $13,$0cc3
0cb7: 2f 0d     bra   $0cc6
0cb9: f5 91 02  mov   a,$0291+x
0cbc: fd        mov   y,a
0cbd: f5 90 02  mov   a,$0290+x
0cc0: 3f 01 12  call  $1201
0cc3: 3f 36 10  call  $1036
0cc6: f2 13     clr7  $13
0cc8: f5 b1 02  mov   a,$02b1+x
0ccb: fd        mov   y,a
0ccc: f5 b0 02  mov   a,$02b0+x
0ccf: da 10     movw  $10,ya
0cd1: f4 90     mov   a,$90+x
0cd3: f0 0e     beq   $0ce3
0cd5: f4 91     mov   a,$91+x
0cd7: d0 0a     bne   $0ce3
0cd9: f5 c1 02  mov   a,$02c1+x
0cdc: fd        mov   y,a
0cdd: f5 c0 02  mov   a,$02c0+x
0ce0: 3f ff 11  call  $11ff
0ce3: f4 a1     mov   a,$a1+x
0ce5: d0 04     bne   $0ceb
0ce7: e3 13 ab  bbs7  $13,$0c95
0cea: 6f        ret
0ceb: f5 40 03  mov   a,$0340+x
0cee: de a0 f6  cbne  $a0+x,$0ce7
0cf1: eb 49     mov   y,$49
0cf3: f5 31 03  mov   a,$0331+x
0cf6: cf        mul   ya
0cf7: dd        mov   a,y
0cf8: 60        clrc
0cf9: 95 30 03  adc   a,$0330+x
0cfc: 5f 70 11  jmp   $1170
0cff: e2 13     set7  $13
0d01: da 16     movw  $16,ya
0d03: cb 12     mov   $12,y
0d05: f3 12 06  bbc7  $12,$0d0e
0d08: ba 0e     movw  ya,$0e
0d0a: 9a 16     subw  ya,$16
0d0c: da 16     movw  $16,ya
0d0e: eb 49     mov   y,$49
0d10: e4 16     mov   a,$16
0d12: cf        mul   ya
0d13: cb 14     mov   $14,y
0d15: 8f 00 15  mov   $15,#$00
0d18: eb 49     mov   y,$49
0d1a: e4 17     mov   a,$17
0d1c: cf        mul   ya
0d1d: 7a 14     addw  ya,$14
0d1f: f3 12 06  bbc7  $12,$0d28
0d22: da 14     movw  $14,ya
0d24: ba 0e     movw  ya,$0e
0d26: 9a 14     subw  ya,$14
0d28: 7a 10     addw  ya,$10
0d2a: da 10     movw  $10,ya
0d2c: 6f        ret
0d2d: e2 13     set7  $13
0d2f: eb 49     mov   y,$49
0d31: f5 62 03  mov   a,$0362+x
0d34: cf        mul   ya
0d35: dd        mov   a,y
0d36: 60        clrc
0d37: 95 60 03  adc   a,$0360+x
0d3a: 1c        asl   a
0d3b: 90 02     bcc   $0d3f
0d3d: 48 ff     eor   a,#$ff
0d3f: fb b1     mov   y,$b1+x
0d41: cf        mul   ya
0d42: f5 11 02  mov   a,$0211+x
0d45: cf        mul   ya
0d46: dd        mov   a,y
0d47: 48 ff     eor   a,#$ff
0d49: 80        setc
0d4a: 95 11 02  adc   a,$0211+x
0d4d: fd        mov   y,a
0d4e: f5 41 02  mov   a,$0241+x
0d51: cf        mul   ya
0d52: e4 57     mov   a,$57
0d54: cf        mul   ya
0d55: dd        mov   a,y
0d56: cf        mul   ya
0d57: dd        mov   a,y
0d58: d5 71 03  mov   $0371+x,a
0d5b: 6f        ret
0d5c: f8 46     mov   x,$46
0d5e: e7 30     mov   a,($30+x)
0d60: bb 30     inc   $30+x
0d62: d0 02     bne   $0d66
0d64: bb 31     inc   $31+x
0d66: fd        mov   y,a
0d67: 6f        ret
0d68: 33 66 80  bbc1  $66,$0ceb
0d6b: 99        adc   (x),(y)
0d6c: b3 cc e6  bbc5  $cc,$0d55
0d6f: ff        stop
0d70: 08 12     or    a,#$12
0d72: 1b 24     asl   $24+x
0d74: 2c 35 3e  rol   $3e35
0d77: 47 51     eor   a,($51+x)
0d79: 5a 62     cmpw  ya,$62
0d7b: 6b 7d     ror   $7d
0d7d: 8f a1 b3  mov   $b3,#$a1
0d80: 00        nop
0d81: 01        tcall 0
0d82: 03 07 0d  bbs0  $07,$0d92
0d85: 15 1e 29  or    a,$291e+x
0d88: 34 42     and   a,$42+x
0d8a: 51        tcall 5
0d8b: 5e 67 6e  cmp   y,$6e67
0d8e: 73 77 7a  bbc3  $77,$0e0b
0d91: 7c        ror   a
0d92: 7d        mov   a,x
0d93: 7e 7f     cmp   y,$7f
0d95: 7f        reti
0d96: 7f        reti
0d97: 00        nop
0d98: 00        nop
0d99: 2f 60     bra   $0dfb
0d9b: 00        nop
0d9c: 00        nop
0d9d: 00        nop
0d9e: 80        setc
0d9f: 60        clrc
0da0: 02 0c     set0  $0c
0da2: 1c        asl   a
0da3: 2c 3c 6c  rol   $6c3c
0da6: 0d        push  psw
0da7: 2d        push  a
0da8: 3d        inc   x
0da9: 4d        push  x
0daa: 5d        mov   x,a
0dab: 6d        push  y
0dac: 7d        mov   a,x
0dad: ff        stop
0dae: 08 17     or    a,#$17
0db0: 24 24     and   a,$24
0db2: 17 08     or    a,($08)+y
0db4: ff        stop
0db5: 7f        reti
0db6: 00        nop
0db7: 00        nop
0db8: 00        nop
0db9: 00        nop
0dba: 00        nop
0dbb: 00        nop
0dbc: 00        nop
0dbd: 8d 00     mov   y,#$00
0dbf: 1c        asl   a
0dc0: cd 18     mov   x,#$18
0dc2: 9e        div   ya,x
0dc3: 5d        mov   x,a
0dc4: f6 da 12  mov   a,$12da+y
0dc7: c4 16     mov   $16,a
0dc9: f6 d9 12  mov   a,$12d9+y
0dcc: 2f 04     bra   $0dd2
0dce: 4b 16     lsr   $16
0dd0: 7c        ror   a
0dd1: 3d        inc   x
0dd2: c8 06     cmp   x,#$06
0dd4: d0 f8     bne   $0dce
0dd6: eb 16     mov   y,$16
0dd8: 6f        ret
0dd9: be        das   a
0dda: 10 bd     bpl   $0d99
0ddc: 11        tcall 1
0ddd: cb 12     mov   $12,y
0ddf: e9 13 18  mov   x,$1813
0de2: 15 59 16  or    a,$1659+x
0de5: ad 17     cmp   y,#$17
0de7: 16 19 94  or    a,$9419+y
0dea: 1a 28     decw  $28
0dec: 1c        asl   a
0ded: d5 1d 9b  mov   $9b1d+x,a
0df0: 1f 00 e8  jmp   ($e800+x)
0df3: aa c5 f4  mov1  c,$14c5,7
0df6: 00        nop
0df7: e8 bb     mov   a,#$bb
0df9: c5 f5 00  mov   $00f5,a
0dfc: e5 f4 00  mov   a,$00f4
0dff: 68 cc     cmp   a,#$cc
0e01: d0 f9     bne   $0dfc
0e03: 2f 20     bra   $0e25
0e05: ec f4 00  mov   y,$00f4
0e08: d0 fb     bne   $0e05
0e0a: 5e f4 00  cmp   y,$00f4
0e0d: d0 0f     bne   $0e1e
0e0f: e5 f5 00  mov   a,$00f5
0e12: cc f4 00  mov   $00f4,y
0e15: d7 14     mov   ($14)+y,a
0e17: fc        inc   y
0e18: d0 f0     bne   $0e0a
0e1a: ab 15     inc   $15
0e1c: 2f ec     bra   $0e0a
0e1e: 10 ea     bpl   $0e0a
0e20: 5e f4 00  cmp   y,$00f4
0e23: 10 e5     bpl   $0e0a
0e25: e5 f6 00  mov   a,$00f6
0e28: ec f7 00  mov   y,$00f7
0e2b: da 14     movw  $14,ya
0e2d: ec f4 00  mov   y,$00f4
0e30: e5 f5 00  mov   a,$00f5
0e33: cc f4 00  mov   $00f4,y
0e36: d0 cd     bne   $0e05
0e38: cd 31     mov   x,#$31
0e3a: c9 f1 00  mov   $00f1,x
0e3d: 6f        ret
0e3e: 00        nop
0e3f: 00        nop
0e40: 00        nop
0e41: 00        nop
0e42: 00        nop
0e43: 00        nop
0e44: 00        nop
0e45: 00        nop
0e46: 00        nop
0e47: 00        nop
0e48: 00        nop
0e49: 00        nop
0e4a: 00        nop
0e4b: 00        nop
0e4c: 00        nop
0e4d: 00        nop
0e4e: 00        nop
0e4f: 00        nop
0e50: 00        nop
0e51: 00        nop
0e52: 00        nop
0e53: 00        nop
0e54: 00        nop
0e55: 00        nop
0e56: 00        nop
0e57: 00        nop
0e58: 00        nop
0e59: 00        nop
0e5a: 00        nop
0e5b: 00        nop
0e5c: 00        nop
0e5d: 00        nop
0e5e: 00        nop
0e5f: 00        nop
0e60: aa 14 18  mov1  c,$1814,0
0e63: 14 42     or    a,$42+x
0e65: 14 6c     or    a,$6c+x
0e67: 14 b0     or    a,$b0+x
0e69: 13 72 13  bbc0  $72,$0e7f
0e6c: dc        dec   y
0e6d: 13 30 15  bbc0  $30,$0e85
0e70: 44 15     eor   a,$15
0e72: 80        setc
0e73: 13 90 13  bbc0  $90,$0e89
0e76: 90 13     bcc   $0e8b
0e78: a0        ei
0e79: 13 ff 00  bbc0  $ff,$0e7c
0e7c: 74 13     cmp   a,$13+x
0e7e: 00        nop
0e7f: 00        nop
0e80: 09 1d 00  or    ($00),($1d)
0e83: 00        nop
0e84: 00        nop
0e85: 00        nop
0e86: 00        nop
0e87: 00        nop
0e88: 00        nop
0e89: 00        nop
0e8a: 00        nop
0e8b: 00        nop
0e8c: 00        nop
0e8d: 00        nop
0e8e: 00        nop
0e8f: 00        nop
0e90: 0d        push  psw
0e91: 1d        dec   x
0e92: 35 1d 4b  and   a,$4b1d+x
0e95: 1d        dec   x
0e96: 63 1d 00  bbs3  $1d,$0e99
0e99: 00        nop
0e9a: 00        nop
0e9b: 00        nop
0e9c: 00        nop
0e9d: 00        nop
0e9e: 75 1d 83  cmp   a,$831d+x
0ea1: 1d        dec   x
0ea2: be        das   a
0ea3: 1d        dec   x
0ea4: d6 1d 00  mov   $001d+y,a
0ea7: 00        nop
0ea8: 00        nop
0ea9: 00        nop
0eaa: 00        nop
0eab: 00        nop
0eac: 00        nop
0ead: 00        nop
0eae: 00        nop
0eaf: 00        nop
0eb0: bc        inc   a
0eb1: 13 bc 13  bbc0  $bc,$0ec7
0eb4: cc 13 ff  mov   $ff13,y
0eb7: 00        nop
0eb8: b0 13     bcs   $0ecd
0eba: 00        nop
0ebb: 00        nop
0ebc: ee        pop   y
0ebd: 1d        dec   x
0ebe: 37 1e     and   a,($1e)+y
0ec0: 20        clrp
0ec1: 1e 62 1e  cmp   x,$1e62
0ec4: 09 1e 4c  or    ($4c),($1e)
0ec7: 1e 00 00  cmp   x,$0000
0eca: 00        nop
0ecb: 00        nop
0ecc: 7e 1e     cmp   y,$1e
0ece: b5 1e a3  sbc   a,$a31e+x
0ed1: 1e d6 1e  cmp   x,$1ed6
0ed4: 91        tcall 9
0ed5: 1e c5 1e  cmp   x,$1ec5
0ed8: 00        nop
0ed9: 00        nop
0eda: 00        nop
0edb: 00        nop
0edc: e8 13     mov   a,#$13
0ede: f8 13     mov   x,$13
0ee0: 08 14     or    a,#$14
0ee2: ff        stop
0ee3: 00        nop
0ee4: de 13 00  cbne  $13+x,$0ee7
0ee7: 00        nop
0ee8: ee        pop   y
0ee9: 1e 4e 1f  cmp   x,$1f4e
0eec: 02 1f     set0  $1f
0eee: bc        inc   a
0eef: 1f 12 1f  jmp   ($1f12+x)
0ef2: 7f        reti
0ef3: 1f 00 00  jmp   ($0000+x)
0ef6: 00        nop
0ef7: 00        nop
0ef8: 22 1f     set1  $1f
0efa: 4e 1f 2a  tclr1 $2a1f
0efd: 1f bc 1f  jmp   ($1fbc+x)
0f00: 31        tcall 3
0f01: 1f 7f 1f  jmp   ($1f7f+x)
0f04: 00        nop
0f05: 00        nop
0f06: 00        nop
0f07: 00        nop
0f08: 38 1f 4e  and   $4e,#$1f
0f0b: 1f 40 1f  jmp   ($1f40+x)
0f0e: bc        inc   a
0f0f: 1f 47 1f  jmp   ($1f47+x)
0f12: 7f        reti
0f13: 1f 00 00  jmp   ($0000+x)
0f16: 00        nop
0f17: 00        nop
0f18: 32 14     clr1  $14
0f1a: 22 14     set1  $14
0f1c: ff        stop
0f1d: 00        nop
0f1e: 18 14 00  or    $00,#$14
0f21: 00        nop
0f22: 19        or    (x),(y)
0f23: 21        tcall 2
0f24: 02 21     set0  $21
0f26: 4e 21 73  tclr1 $7321
0f29: 21        tcall 2
0f2a: 00        nop
0f2b: 00        nop
0f2c: 00        nop
0f2d: 00        nop
0f2e: 00        nop
0f2f: 00        nop
0f30: 39        and   (x),(y)
0f31: 22 b9     set1  $b9
0f33: 21        tcall 2
0f34: 9f        xcn   a
0f35: 21        tcall 2
0f36: e9 21 11  mov   x,$1121
0f39: 22 00     set1  $00
0f3b: 00        nop
0f3c: 00        nop
0f3d: 00        nop
0f3e: 00        nop
0f3f: 00        nop
0f40: 39        and   (x),(y)
0f41: 22 4c     set1  $4c
0f43: 14 5c     or    a,$5c+x
0f45: 14 ff     or    a,$ff+x
0f47: 00        nop
0f48: 42 14     set2  $14
0f4a: 00        nop
0f4b: 00        nop
0f4c: d6 1f 15  mov   $151f+y,a
0f4f: 20        clrp
0f50: 50 20     bvc   $0f72
0f52: 77 20     cmp   a,($20)+y
0f54: 00        nop
0f55: 00        nop
0f56: 00        nop
0f57: 00        nop
0f58: 00        nop
0f59: 00        nop
0f5a: 00        nop
0f5b: 00        nop
0f5c: a7 20     sbc   a,($20+x)
0f5e: c2 20     set6  $20
0f60: da 20     movw  $20,ya
0f62: ed        notc
0f63: 20        clrp
0f64: 00        nop
0f65: 00        nop
0f66: 00        nop
0f67: 00        nop
0f68: 00        nop
0f69: 00        nop
0f6a: 00        nop
0f6b: 00        nop
0f6c: 7a 14     addw  ya,$14
0f6e: 8a 14 8a  eor1  c,$0a14,4
0f71: 14 9a     or    a,$9a+x
0f73: 14 ff     or    a,$ff+x
0f75: 00        nop
0f76: 6e 14 00  dbnz  $14,$0f79
0f79: 00        nop
0f7a: 00        nop
0f7b: 00        nop
0f7c: 00        nop
0f7d: 00        nop
0f7e: 48 23     eor   a,#$23
0f80: b0 23     bcs   $0fa5
0f82: 63 23 00  bbs3  $23,$0f85
0f85: 00        nop
0f86: 00        nop
0f87: 00        nop
0f88: d3 23 52  bbc6  $23,$0fdd
0f8b: 22 76     set1  $76
0f8d: 22 48     set1  $48
0f8f: 23 b0 23  bbs1  $b0,$0fb5
0f92: 63 23 bc  bbs3  $23,$0f51
0f95: 22 99     set1  $99
0f97: 22 d3     set1  $d3
0f99: 23 df 22  bbs1  $df,$0fbe
0f9c: fa 22 7b  mov   ($7b),($22)
0f9f: 23 b0 23  bbs1  $b0,$0fc5
0fa2: 8e        pop   psw
0fa3: 23 2e 23  bbs1  $2e,$0fc9
0fa6: 14 23     or    a,$23+x
0fa8: 09 24 c0  or    ($c0),($24)
0fab: 14 d0     or    a,$d0+x
0fad: 14 e0     or    a,$e0+x
0faf: 14 f0     or    a,$f0+x
0fb1: 14 00     or    a,$00+x
0fb3: 15 10 15  or    a,$1510+x
0fb6: 10 15     bpl   $0fcd
0fb8: 20        clrp
0fb9: 15 ff 00  or    a,$00ff+x
0fbc: ac 14 00  inc   $0014
0fbf: 00        nop
0fc0: 48 24     eor   a,#$24
0fc2: 71        tcall 7
0fc3: 24 00     and   a,$00
0fc5: 00        nop
0fc6: 95 24 af  adc   a,$af24+x
0fc9: 24 00     and   a,$00
0fcb: 00        nop
0fcc: 00        nop
0fcd: 00        nop
0fce: 00        nop
0fcf: 00        nop
0fd0: c3 24 f9  bbs6  $24,$0fcc
0fd3: 24 6d     and   a,$6d
0fd5: 25 47 25  and   a,$2547
0fd8: 93 25 00  bbc4  $25,$0fdb
0fdb: 00        nop
0fdc: 00        nop
0fdd: 00        nop
0fde: 00        nop
0fdf: 00        nop
0fe0: d7 25     mov   ($25)+y,a
0fe2: 00        nop
0fe3: 26        and   a,(x)
0fe4: 65 26 3f  cmp   a,$3f26
0fe7: 26        and   a,(x)
0fe8: 8b 26     dec   $26
0fea: 00        nop
0feb: 00        nop
0fec: b9        sbc   (x),(y)
0fed: 25 00 00  and   a,$0000
0ff0: b0 26     bcs   $1018
0ff2: cc 26 51  mov   $5126,y
0ff5: 27 2a     and   a,($2a+x)
0ff7: 27 00     and   a,($00+x)
0ff9: 00        nop
0ffa: 00        nop
0ffb: 00        nop
0ffc: 00        nop
0ffd: 00        nop
0ffe: 00        nop
0fff: 00        nop
1000: 65 27 85  cmp   a,$8527
1003: 27 09     and   a,($09+x)
1005: 28 e3     and   a,#$e3
1007: 27 00     and   a,($00+x)
1009: 00        nop
100a: 00        nop
100b: 00        nop
100c: 00        nop
100d: 00        nop
100e: 00        nop
100f: 00        nop
1010: 1d        dec   x
1011: 28 42     and   a,#$42
1013: 28 7e     and   a,#$7e
1015: 28 6d     and   a,#$6d
1017: 28 00     and   a,#$00
1019: 00        nop
101a: 00        nop
101b: 00        nop
101c: 00        nop
101d: 00        nop
101e: 00        nop
101f: 00        nop
1020: 8f 28 ce  mov   $ce,#$28
1023: 28 57     and   a,#$57
1025: 29 0c 29  and   ($29),($0c)
1028: 31        tcall 3
1029: 29 00 00  and   ($00),($00)
102c: 00        nop
102d: 00        nop
102e: 00        nop
102f: 00        nop
1030: 34 15     and   a,$15+x
1032: 00        nop
1033: 00        nop
1034: 51        tcall 5
1035: 1c        asl   a
1036: 93 1c b2  bbc4  $1c,$0feb
1039: 1c        asl   a
103a: d1        tcall 13
103b: 1c        asl   a
103c: 00        nop
103d: 00        nop
103e: 74 1c     cmp   a,$1c+x
1040: 00        nop
1041: 00        nop
1042: ef        sleep
1043: 1c        asl   a
1044: be        das   a
1045: 15 ce 15  or    a,$15ce+x
1048: de 15 be  cbne  $15+x,$1009
104b: 15 ce 15  or    a,$15ce+x
104e: de 15 be  cbne  $15+x,$100f
1051: 15 ce 15  or    a,$15ce+x
1054: de 15 be  cbne  $15+x,$1015
1057: 15 ce 15  or    a,$15ce+x
105a: de 15 be  cbne  $15+x,$101b
105d: 15 ce 15  or    a,$15ce+x
1060: de 15 be  cbne  $15+x,$1021
1063: 15 ce 15  or    a,$15ce+x
1066: de 15 be  cbne  $15+x,$1027
1069: 15 ce 15  or    a,$15ce+x
106c: de 15 be  cbne  $15+x,$102d
106f: 15 ce 15  or    a,$15ce+x
1072: de 15 be  cbne  $15+x,$1033
1075: 15 ee 15  or    a,$15ee+x
1078: fe 15     dbnz  y,$108f
107a: fe 15     dbnz  y,$1091
107c: 0e 16 1e  tset1 $1e16
107f: 16 0e 16  or    a,$160e+y
1082: 2e 16 0e  cbne  $16,$1093
1085: 16 1e 16  or    a,$161e+y
1088: 0e 16 2e  tset1 $2e16
108b: 16 3e 16  or    a,$163e+y
108e: 4e 16 3e  tclr1 $3e16
1091: 16 5e 16  or    a,$165e+y
1094: fe 15     dbnz  y,$10ab
1096: fe 15     dbnz  y,$10ad
1098: 6e 16 7e  dbnz  $16,$1119
109b: 16 6e 16  or    a,$166e+y
109e: 8e        pop   psw
109f: 16 6e 16  or    a,$166e+y
10a2: 7e 16     cmp   y,$16
10a4: 6e 16 8e  dbnz  $16,$1035
10a7: 16 3e 16  or    a,$163e+y
10aa: 4e 16 3e  tclr1 $3e16
10ad: 16 5e 16  or    a,$165e+y
10b0: 6e 16 7e  dbnz  $16,$1131
10b3: 16 6e 16  or    a,$166e+y
10b6: 8e        pop   psw
10b7: 16 ff 00  or    a,$00ff+y
10ba: 78 15 00  cmp   $00,#$15
10bd: 00        nop
10be: 6a 19 62  and1  c,!($0219,3)
10c1: 17 3b     or    a,($3b)+y
10c3: 17 26     or    a,($26)+y
10c5: 18 95 19  or    $19,#$95
10c8: 1f 19 40  jmp   ($4019+x)
10cb: 19        or    (x),(y)
10cc: f9 18     mov   x,$18+y
10ce: 6a 19 b0  and1  c,!($1019,5)
10d1: 17 89     or    a,($89)+y
10d3: 17 94     or    a,($94)+y
10d5: 18 95 19  or    $19,#$95
10d8: 1f 19 40  jmp   ($4019+x)
10db: 19        or    (x),(y)
10dc: f9 18     mov   x,$18+y
10de: 29 1a ed  and   ($ed),($1a)
10e1: 16 c6 16  or    a,$16c6+y
10e4: 4a 18 51  and1  c,$1118,2
10e7: 1a e1     decw  $e1
10e9: 19        or    (x),(y)
10ea: 03 1a bb  bbs0  $1a,$10a8
10ed: 19        or    (x),(y)
10ee: 8a 1b 79  eor1  c,$191b,3
10f1: 1b 4e     asl   $4e+x
10f3: 1b 64     asl   $64+x
10f5: 1b 95     asl   $95+x
10f7: 19        or    (x),(y)
10f8: 1f 19 6a  jmp   ($6a19+x)
10fb: 19        or    (x),(y)
10fc: f9 18     mov   x,$18+y
10fe: 6e 18 14  dbnz  $18,$1115
1101: 17 9e     or    a,($9e)+y
1103: 16 d7 17  or    a,$17d7+y
1106: 95 19 1f  adc   a,$1f19+x
1109: 19        or    (x),(y)
110a: 6a 19 f9  and1  c,!($1919,7)
110d: 18 6a 19  or    $19,#$6a
1110: cf        mul   ya
1111: 18 b8 18  or    $18,#$b8
1114: e5 18 95  mov   a,$9518
1117: 19        or    (x),(y)
1118: 1f 19 00  jmp   ($0019+x)
111b: 00        nop
111c: f9 18     mov   x,$18+y
111e: 6a 19 8d  and1  c,!($0d19,4)
1121: 1a 77     decw  $77
1123: 1a e5     decw  $e5
1125: 18 95 19  or    $19,#$95
1128: 1f 19 00  jmp   ($0019+x)
112b: 00        nop
112c: f9 18     mov   x,$18+y
112e: 6a 19 b6  and1  c,!($1619,5)
1131: 1a a2     decw  $a2
1133: 1a c9     decw  $c9
1135: 1a 95     decw  $95
1137: 19        or    (x),(y)
1138: 1f 19 00  jmp   ($0019+x)
113b: 00        nop
113c: f9 18     mov   x,$18+y
113e: 6a 19 3f  and1  c,!($1f19,1)
1141: 1b dd     asl   $dd+x
1143: 1a f2     decw  $f2
1145: 1a 95     decw  $95
1147: 19        or    (x),(y)
1148: 1f 19 06  jmp   ($0619+x)
114b: 1b f9     asl   $f9+x
114d: 18 6a 19  or    $19,#$6a
1150: 3f 1b 1c  call  $1c1b
1153: 1b 2e     asl   $2e+x
1155: 1b 95     asl   $95+x
1157: 19        or    (x),(y)
1158: 1f 19 06  jmp   ($0619+x)
115b: 1b f9     asl   $f9+x
115d: 18 6a 19  or    $19,#$6a
1160: 79        cmp   (x),(y)
1161: 1b 4e     asl   $4e+x
1163: 1b 64     asl   $64+x
1165: 1b 95     asl   $95+x
1167: 19        or    (x),(y)
1168: 1f 19 8a  jmp   ($8a19+x)
116b: 1b f9     asl   $f9+x
116d: 18 6a 19  or    $19,#$6a
1170: c6        mov   (x),a
1171: 1b 9d     asl   $9d+x
1173: 1b b2     asl   $b2+x
1175: 1b 95     asl   $95+x
1177: 19        or    (x),(y)
1178: 1f 19 00  jmp   ($0019+x)
117b: 00        nop
117c: f9 18     mov   x,$18+y
117e: 6a 19 02  and1  c,!($0219,0)
1181: 1c        asl   a
1182: db 1b     mov   $1b+x,y
1184: ef        sleep
1185: 1b 95     asl   $95+x
1187: 19        or    (x),(y)
1188: 1f 19 00  jmp   ($0019+x)
118b: 00        nop
118c: f9 18     mov   x,$18+y
118e: 6a 19 3e  and1  c,!($1e19,1)
1191: 1c        asl   a
1192: 17 1c     or    a,($1c)+y
1194: 29 1c 95  and   ($95),($1c)
1197: 19        or    (x),(y)
1198: 1f 19 00  jmp   ($0019+x)
119b: 00        nop
119c: f9 18     mov   x,$18+y
119e: da 04     movw  $04,ya
11a0: db 0a     mov   $0a+x,y
11a2: 0c 6c b0  asl   $b06c
11a5: c7 c7     mov   ($c7+x),a
11a7: ab c7     inc   $c7
11a9: c7 a8     mov   ($a8+x),a
11ab: c7 c7     mov   ($c7+x),a
11ad: ad c7     cmp   y,#$c7
11af: 18 af 0c  or    $0c,#$af
11b2: ae        pop   a
11b3: ad c7     cmp   y,#$c7
11b5: 10 ab     bpl   $1162
11b7: b4 b7     sbc   a,$b7+x
11b9: 0c b9 c7  asl   $c7b9
11bc: b5 b7 c7  sbc   a,$c7b7+x
11bf: b4 c7     sbc   a,$c7+x
11c1: b0 b2     bcs   $1175
11c3: 24 af     and   a,$af
11c5: 00        nop
11c6: da 03     movw  $03,ya
11c8: db 0a     mov   $0a+x,y
11ca: 0c 6c a8  asl   $a86c
11cd: c7 a8     mov   ($a8+x),a
11cf: a8 c7     sbc   a,#$c7
11d1: a8 c7     sbc   a,#$c7
11d3: a8 c7     sbc   a,#$c7
11d5: a8 c7     sbc   a,#$c7
11d7: a8 c7     sbc   a,#$c7
11d9: a8 a8     sbc   a,#$a8
11db: a8 0c     sbc   a,#$0c
11dd: a8 c7     sbc   a,#$c7
11df: a8 c7     sbc   a,#$c7
11e1: c7 a9     mov   ($a9+x),a
11e3: c7 a9     mov   ($a9+x),a
11e5: a8 c7     sbc   a,#$c7
11e7: c7 c7     mov   ($c7+x),a
11e9: c7 c7     mov   ($c7+x),a
11eb: c7 c7     mov   ($c7+x),a
11ed: da 03     movw  $03,ya
11ef: db 0a     mov   $0a+x,y
11f1: 0c 6c b0  asl   $b06c
11f4: c7 b4     mov   ($b4+x),a
11f6: af        mov   (x)+,a
11f7: c7 b4     mov   ($b4+x),a
11f9: c7 ad     mov   ($ad+x),a
11fb: c7 b4     mov   ($b4+x),a
11fd: c7 af     mov   ($af+x),a
11ff: c7 af     mov   ($af+x),a
1201: b0 b2     bcs   $11b5
1203: 0c b0 c7  asl   $c7b0
1206: b0 c7     bcs   $11cf
1208: c7 b0     mov   ($b0+x),a
120a: c7 b0     mov   ($b0+x),a
120c: b0 c7     bcs   $11d5
120e: c7 c7     mov   ($c7+x),a
1210: c7 c7     mov   ($c7+x),a
1212: c7 c7     mov   ($c7+x),a
1214: da 03     movw  $03,ya
1216: db 14     mov   $14+x,y
1218: 0c 69 b0  asl   $b069
121b: c7 b4     mov   ($b4+x),a
121d: af        mov   (x)+,a
121e: c7 b4     mov   ($b4+x),a
1220: c7 ad     mov   ($ad+x),a
1222: c7 b4     mov   ($b4+x),a
1224: c7 af     mov   ($af+x),a
1226: c7 af     mov   ($af+x),a
1228: b0 b2     bcs   $11dc
122a: 0c b0 c7  asl   $c7b0
122d: b4 af     sbc   a,$af+x
122f: c7 b4     mov   ($b4+x),a
1231: c7 ad     mov   ($ad+x),a
1233: c7 b4     mov   ($b4+x),a
1235: c7 af     mov   ($af+x),a
1237: c7 af     mov   ($af+x),a
1239: b0 b2     bcs   $11ed
123b: da 03     movw  $03,ya
123d: db 0a     mov   $0a+x,y
123f: 0c 6c ab  asl   $ab6c
1242: c7 ab     mov   ($ab+x),a
1244: ab c7     inc   $c7
1246: ab c7     inc   $c7
1248: ab c7     inc   $c7
124a: ab c7     inc   $c7
124c: ab c7     inc   $c7
124e: ab ab     inc   $ab
1250: ab 0c     inc   $0c
1252: ab c7     inc   $c7
1254: ab ab     inc   $ab
1256: c7 ab     mov   ($ab+x),a
1258: c7 ab     mov   ($ab+x),a
125a: c7 ab     mov   ($ab+x),a
125c: c7 ab     mov   ($ab+x),a
125e: c7 ab     mov   ($ab+x),a
1260: ab ab     inc   $ab
1262: da 03     movw  $03,ya
1264: db 0a     mov   $0a+x,y
1266: 0c 6c b0  asl   $b06c
1269: c7 b4     mov   ($b4+x),a
126b: af        mov   (x)+,a
126c: c7 b4     mov   ($b4+x),a
126e: c7 ad     mov   ($ad+x),a
1270: c7 b4     mov   ($b4+x),a
1272: c7 af     mov   ($af+x),a
1274: c7 af     mov   ($af+x),a
1276: b0 b2     bcs   $122a
1278: 0c b0 c7  asl   $c7b0
127b: b4 af     sbc   a,$af+x
127d: c7 b4     mov   ($b4+x),a
127f: c7 ad     mov   ($ad+x),a
1281: c7 b4     mov   ($b4+x),a
1283: c7 af     mov   ($af+x),a
1285: c7 af     mov   ($af+x),a
1287: b0 b1     bcs   $123a
1289: da 03     movw  $03,ya
128b: db 0a     mov   $0a+x,y
128d: 0c 6c ad  asl   $ad6c
1290: c7 ad     mov   ($ad+x),a
1292: ad c7     cmp   y,#$c7
1294: ad c7     cmp   y,#$c7
1296: ad c7     cmp   y,#$c7
1298: ad c7     cmp   y,#$c7
129a: ad c7     cmp   y,#$c7
129c: ad ad     cmp   y,#$ad
129e: ad 0c     cmp   y,#$0c
12a0: ad c7     cmp   y,#$c7
12a2: ad ad     cmp   y,#$ad
12a4: c7 ad     mov   ($ad+x),a
12a6: c7 ad     mov   ($ad+x),a
12a8: c7 ad     mov   ($ad+x),a
12aa: c7 ad     mov   ($ad+x),a
12ac: c7 ad     mov   ($ad+x),a
12ae: ad ad     cmp   y,#$ad
12b0: da 03     movw  $03,ya
12b2: db 0a     mov   $0a+x,y
12b4: 0c 6c b2  asl   $b26c
12b7: c7 b5     mov   ($b5+x),a
12b9: b1        tcall 11
12ba: c7 b5     mov   ($b5+x),a
12bc: c7 b0     mov   ($b0+x),a
12be: c7 b5     mov   ($b5+x),a
12c0: c7 af     mov   ($af+x),a
12c2: c7 af     mov   ($af+x),a
12c4: b0 b1     bcs   $1277
12c6: 0c b2 c7  asl   $c7b2
12c9: b5 b1 c7  sbc   a,$c7b1+x
12cc: b5 c7 b0  sbc   a,$b0c7+x
12cf: c7 b5     mov   ($b5+x),a
12d1: c7 af     mov   ($af+x),a
12d3: c7 af     mov   ($af+x),a
12d5: b0 b2     bcs   $1289
12d7: da 04     movw  $04,ya
12d9: db 0a     mov   $0a+x,y
12db: 0c 6c a8  asl   $a86c
12de: c7 c7     mov   ($c7+x),a
12e0: a4 c7     sbc   a,$c7
12e2: c7 9f     mov   ($9f+x),a
12e4: c7 c7     mov   ($c7+x),a
12e6: a4 c7     sbc   a,$c7
12e8: 18 a6 0c  or    $0c,#$a6
12eb: a5 a4 c7  sbc   a,$c7a4
12ee: 10 a4     bpl   $1294
12f0: ab af     inc   $af
12f2: 0c b0 c7  asl   $c7b0
12f5: ad af     cmp   y,#$af
12f7: c7 ab     mov   ($ab+x),a
12f9: c7 a8     mov   ($a8+x),a
12fb: a9 24 a6  sbc   ($a6),($24)
12fe: da 04     movw  $04,ya
1300: 0c 6c 9f  asl   $9f6c
1303: c7 c7     mov   ($c7+x),a
1305: 9c        dec   a
1306: c7 c7     mov   ($c7+x),a
1308: 98 c7 c7  adc   $c7,#$c7
130b: 9d        mov   x,sp
130c: c7 18     mov   ($18+x),a
130e: 9f        xcn   a
130f: 0c 9e 9d  asl   $9d9e
1312: c7 60     mov   ($60+x),a
1314: c7 c7     mov   ($c7+x),a
1316: 10 9c     bpl   $12b4
1318: a4 a8     sbc   a,$a8
131a: 0c a9 c7  asl   $c7a9
131d: a6        sbc   a,(x)
131e: a8 c7     sbc   a,#$c7
1320: a4 c7     sbc   a,$c7
1322: a1        tcall 10
1323: a3 24 9f  bbs5  $24,$12c5
1326: da 0e     movw  $0e,ya
1328: 0c 6d 8c  asl   $8c6d
132b: c6        mov   (x),a
132c: c6        mov   (x),a
132d: 93 93 c6  bbc4  $93,$12f6
1330: c6        mov   (x),a
1331: 98 98 c6  adc   $c6,#$98
1334: c6        mov   (x),a
1335: 93 93 c6  bbc4  $93,$12fe
1338: 93 c6 8c  bbc4  $c6,$12c7
133b: c6        mov   (x),a
133c: c6        mov   (x),a
133d: 93 93 c6  bbc4  $93,$1306
1340: c6        mov   (x),a
1341: 98 c6 98  adc   $98,#$c6
1344: c6        mov   (x),a
1345: 98 93 c6  adc   $c6,#$93
1348: 93 c6 da  bbc4  $c6,$1325
134b: 0e 0c 6d  tset1 $6d0c
134e: 8c c6 c6  dec   $c6c6
1351: 93 93 c6  bbc4  $93,$131a
1354: c6        mov   (x),a
1355: 98 98 c6  adc   $c6,#$98
1358: c6        mov   (x),a
1359: 93 93 c6  bbc4  $93,$1322
135c: 93 c6 8c  bbc4  $c6,$12eb
135f: c6        mov   (x),a
1360: 8c c6 c6  dec   $c6c6
1363: 8d c6     mov   y,#$c6
1365: 8d 8c     mov   y,#$8c
1367: c6        mov   (x),a
1368: c6        mov   (x),a
1369: 93 93 c6  bbc4  $93,$1332
136c: 93 c6 da  bbc4  $c6,$1349
136f: 0e 0c 6d  tset1 $6d0c
1372: db 0a     mov   $0a+x,y
1374: 8c c6 c6  dec   $c6c6
1377: 8c 90 c6  dec   $c690
137a: c6        mov   (x),a
137b: 90 91     bcc   $130e
137d: c6        mov   (x),a
137e: c6        mov   (x),a
137f: 91        tcall 9
1380: 92 c6     clr4  $c6
1382: 93 c6 8c  bbc4  $c6,$1311
1385: c6        mov   (x),a
1386: c6        mov   (x),a
1387: 8c 90 c6  dec   $c690
138a: c6        mov   (x),a
138b: 90 91     bcc   $131e
138d: c6        mov   (x),a
138e: c6        mov   (x),a
138f: 91        tcall 9
1390: 92 c6     clr4  $c6
1392: 93 c6 da  bbc4  $c6,$136f
1395: 0e 0c 6d  tset1 $6d0c
1398: 8e        pop   psw
1399: c6        mov   (x),a
139a: c6        mov   (x),a
139b: 95 95 c6  adc   a,$c695+x
139e: c6        mov   (x),a
139f: 9a 9a     subw  ya,$9a
13a1: c6        mov   (x),a
13a2: c6        mov   (x),a
13a3: 95 95 c6  adc   a,$c695+x
13a6: 95 c6 8e  adc   a,$8ec6+x
13a9: c6        mov   (x),a
13aa: c6        mov   (x),a
13ab: 95 95 c6  adc   a,$c695+x
13ae: c6        mov   (x),a
13af: 9a c6     subw  ya,$c6
13b1: 9a c6     subw  ya,$c6
13b3: 9a 95     subw  ya,$95
13b5: c6        mov   (x),a
13b6: 95 c6 da  adc   a,$dac6+x
13b9: 07 db     or    a,($db+x)
13bb: 0a 18 6b  or1   c,$0b18,3
13be: c7 0c     mov   ($0c+x),a
13c0: b7 b6     sbc   a,($b6)+y
13c2: b5 b3 c7  sbc   a,$c7b3+x
13c5: b4 c7     sbc   a,$c7+x
13c7: ac ad b0  inc   $b0ad
13ca: c7 ad     mov   ($ad+x),a
13cc: b0 b2     bcs   $1380
13ce: 00        nop
13cf: da 07     movw  $07,ya
13d1: db 0a     mov   $0a+x,y
13d3: 18 6b c7  or    $c7,#$6b
13d6: 0c b4 b3  asl   $b3b4
13d9: b2 af     clr5  $af
13db: c7 b0     mov   ($b0+x),a
13dd: c7 a8     mov   ($a8+x),a
13df: a9 ab c7  sbc   ($c7),($ab)
13e2: a4 a8     sbc   a,$a8
13e4: a9 da 0e  sbc   ($0e),($da)
13e7: 0c 6d 8c  asl   $8c6d
13ea: c6        mov   (x),a
13eb: c6        mov   (x),a
13ec: 8c 90 c6  dec   $c690
13ef: c6        mov   (x),a
13f0: 90 91     bcc   $1383
13f2: c6        mov   (x),a
13f3: c6        mov   (x),a
13f4: 91        tcall 9
13f5: 95 c6 98  adc   a,$98c6+x
13f8: c6        mov   (x),a
13f9: da 00     movw  $00,ya
13fb: db 14     mov   $14+x,y
13fd: 0c 7c d2  asl   $d27c
1400: c6        mov   (x),a
1401: d2 d2     clr6  $d2
1403: d2 c6     clr6  $c6
1405: d2 d2     clr6  $d2
1407: d2 c6     clr6  $c6
1409: d2 d2     clr6  $d2
140b: d2 c6     clr6  $c6
140d: d2 d2     clr6  $d2
140f: d2 c6     clr6  $c6
1411: d2 d2     clr6  $d2
1413: d2 c6     clr6  $c6
1415: d2 d2     clr6  $d2
1417: d2 c6     clr6  $c6
1419: d2 d2     clr6  $d2
141b: d2 c6     clr6  $c6
141d: d2 d2     clr6  $d2
141f: da 0c     movw  $0c,ya
1421: db 05     mov   $05+x,y
1423: 18 7c a9  or    $a9,#$7c
1426: a9 b0 0c  sbc   ($0c),($b0)
1429: c6        mov   (x),a
142a: a9 24 b0  sbc   ($b0),($24)
142d: 0c a9 18  asl   $18a9
1430: a9 a9 18  sbc   ($18),($a9)
1433: a9 a9 b0  sbc   ($b0),($a9)
1436: 0c c6 a9  asl   $a9c6
1439: 24 b0     and   a,$b0
143b: 0c a9 18  asl   $18a9
143e: a9 a9 da  sbc   ($da),($a9)
1441: 02 db     set0  $db
1443: 00        nop
1444: de 00 00  cbne  $00+x,$1447
1447: 00        nop
1448: 0c 15 ab  asl   $ab15
144b: ab c6     inc   $c6
144d: b2 ab     clr5  $ab
144f: ab c6     inc   $c6
1451: b2 c7     clr5  $c7
1453: b2 c7     clr5  $c7
1455: b2 ab     clr5  $ab
1457: ab c6     inc   $c6
1459: b2 ab     clr5  $ab
145b: ab c6     inc   $c6
145d: b2 ab     clr5  $ab
145f: ab c6     inc   $c6
1461: b2 c7     clr5  $c7
1463: b2 c7     clr5  $c7
1465: b2 ab     clr5  $ab
1467: ab c6     inc   $c6
1469: b2 f0     clr5  $f0
146b: da 0c     movw  $0c,ya
146d: db 0c     mov   $0c+x,y
146f: e2 29     set7  $29
1471: 18 6c 98  or    $98,#$6c
1474: 9f        xcn   a
1475: 0c 9f 98  asl   $989f
1478: c6        mov   (x),a
1479: 18 98 9f  or    $9f,#$98
147c: 0c 9f 0c  asl   $0c9f
147f: 9f        xcn   a
1480: c6        mov   (x),a
1481: 98 98 18  adc   $18,#$98
1484: 98 9f 0c  adc   $0c,#$9f
1487: 9f        xcn   a
1488: 98 c6 18  adc   $18,#$c6
148b: 98 9f 0c  adc   $0c,#$9f
148e: 9f        xcn   a
148f: 0c 9f c6  asl   $c69f
1492: 98 98 00  adc   $00,#$98
1495: da 00     movw  $00,ya
1497: db 0a     mov   $0a+x,y
1499: 0c 6c d6  asl   $d66c
149c: d6 c7 d3  mov   $d3c7+y,a
149f: d3 d3 c7  bbc6  $d3,$1469
14a2: d6 c7 d6  mov   $d6c7+y,a
14a5: d6 c7 d3  mov   $d3c7+y,a
14a8: d3 c7 d6  bbc6  $c7,$1481
14ab: d6 d6 c7  mov   $c7d6+y,a
14ae: d3 d3 d3  bbc6  $d3,$1484
14b1: c7 d6     mov   ($d6+x),a
14b3: c7 d6     mov   ($d6+x),a
14b5: d6 c7 d3  mov   $d3c7+y,a
14b8: d3 c7 d6  bbc6  $c7,$1491
14bb: da 00     movw  $00,ya
14bd: db 14     mov   $14+x,y
14bf: 0c 7c d2  asl   $d27c
14c2: c6        mov   (x),a
14c3: d2 d2     clr6  $d2
14c5: d2 c6     clr6  $c6
14c7: d2 d2     clr6  $d2
14c9: d2 c6     clr6  $c6
14cb: d2 d2     clr6  $d2
14cd: d2 c6     clr6  $c6
14cf: d2 d2     clr6  $d2
14d1: d2 d2     clr6  $d2
14d3: d2 c6     clr6  $c6
14d5: d2 d2     clr6  $d2
14d7: c6        mov   (x),a
14d8: d2 d2     clr6  $d2
14da: c6        mov   (x),a
14db: c6        mov   (x),a
14dc: c6        mov   (x),a
14dd: d2 c6     clr6  $c6
14df: d2 d2     clr6  $d2
14e1: da 0c     movw  $0c,ya
14e3: db 05     mov   $05+x,y
14e5: 18 7c a9  or    $a9,#$7c
14e8: a9 b0 0c  sbc   ($0c),($b0)
14eb: c6        mov   (x),a
14ec: a9 24 b0  sbc   ($b0),($24)
14ef: 0c a9 18  asl   $18a9
14f2: a9 a9 18  sbc   ($18),($a9)
14f5: b0 b0     bcs   $14a7
14f7: 0c c6 b0  asl   $b0c6
14fa: c6        mov   (x),a
14fb: b0 24     bcs   $1521
14fd: b0 0c     bcs   $150b
14ff: a4 18     sbc   a,$18
1501: a4 a4     sbc   a,$a4
1503: da 02     movw  $02,ya
1505: db 00     mov   $00+x,y
1507: 0c 15 ab  asl   $ab15
150a: ab c6     inc   $c6
150c: b2 ab     clr5  $ab
150e: ab c6     inc   $c6
1510: b2 c7     clr5  $c7
1512: b2 c7     clr5  $c7
1514: b2 ab     clr5  $ab
1516: ab c6     inc   $c6
1518: b2 b2     clr5  $b2
151a: c7 b2     mov   ($b2+x),a
151c: c7 c7     mov   ($c7+x),a
151e: b2 c7     clr5  $c7
1520: b2 b2     clr5  $b2
1522: c7 c7     mov   ($c7+x),a
1524: c7 b2     mov   ($b2+x),a
1526: b2 c6     clr5  $c6
1528: ab da     inc   $da
152a: 0c db 0c  asl   $0cdb
152d: 18 6c 98  or    $98,#$6c
1530: 9f        xcn   a
1531: 0c 9f 98  asl   $989f
1534: c6        mov   (x),a
1535: 18 98 9f  or    $9f,#$98
1538: 0c 9f 0c  asl   $0c9f
153b: 9f        xcn   a
153c: c6        mov   (x),a
153d: 98 98 18  adc   $18,#$98
1540: 6c 9f 9f  ror   $9f9f
1543: 0c c6 9f  asl   $9fc6
1546: c6        mov   (x),a
1547: 9f        xcn   a
1548: 18 98 c6  or    $c6,#$98
154b: 0c 9f 9f  asl   $9f9f
154e: c6        mov   (x),a
154f: 98 00 da  adc   $da,#$00
1552: 00        nop
1553: db 0a     mov   $0a+x,y
1555: 0c 6c d6  asl   $d66c
1558: d6 c7 d3  mov   $d3c7+y,a
155b: d3 d3 c7  bbc6  $d3,$1525
155e: d6 c7 d6  mov   $d6c7+y,a
1561: d6 c7 d3  mov   $d3c7+y,a
1564: d3 c7 d6  bbc6  $c7,$153d
1567: d3 c7 d3  bbc6  $c7,$153d
156a: c7 c7     mov   ($c7+x),a
156c: d3 c7 d3  bbc6  $c7,$1542
156f: d3 c7 c7  bbc6  $c7,$1539
1572: c7 d3     mov   ($d3+x),a
1574: d3 c7 d6  bbc6  $c7,$154d
1577: da 07     movw  $07,ya
1579: db 0a     mov   $0a+x,y
157b: 18 6b c7  or    $c7,#$6b
157e: 0c b7 b6  asl   $b6b7
1581: b5 b3 c7  sbc   a,$c7b3+x
1584: b4 c7     sbc   a,$c7+x
1586: bc        inc   a
1587: c7 bc     mov   ($bc+x),a
1589: 18 bc c7  or    $c7,#$bc
158c: 00        nop
158d: da 07     movw  $07,ya
158f: db 0a     mov   $0a+x,y
1591: 18 6b c7  or    $c7,#$6b
1594: 0c b4 b3  asl   $b3b4
1597: b2 af     clr5  $af
1599: c7 b0     mov   ($b0+x),a
159b: c7 b5     mov   ($b5+x),a
159d: c7 b5     mov   ($b5+x),a
159f: 18 b5 c7  or    $c7,#$b5
15a2: da 07     movw  $07,ya
15a4: db 0a     mov   $0a+x,y
15a6: 18 6b c7  or    $c7,#$6b
15a9: 0c b3 c7  asl   $c7b3
15ac: c7 b2     mov   ($b2+x),a
15ae: c7 c7     mov   ($c7+x),a
15b0: 18 b0 c6  or    $c6,#$b0
15b3: c6        mov   (x),a
15b4: c7 00     mov   ($00+x),a
15b6: da 07     movw  $07,ya
15b8: db 0a     mov   $0a+x,y
15ba: 18 6b c7  or    $c7,#$6b
15bd: 0c ac c7  asl   $c7ac
15c0: c7 a9     mov   ($a9+x),a
15c2: c7 c7     mov   ($c7+x),a
15c4: 18 a8 c6  or    $c6,#$a8
15c7: c6        mov   (x),a
15c8: c7 da     mov   ($da+x),a
15ca: 0e 0c 6d  tset1 $6d0c
15cd: 94 c6     adc   a,$c6+x
15cf: c6        mov   (x),a
15d0: 94 96     adc   a,$96+x
15d2: c6        mov   (x),a
15d3: c6        mov   (x),a
15d4: 96 98 95  adc   a,$9598+y
15d7: 96 97 98  adc   a,$9897+y
15da: c6        mov   (x),a
15db: c6        mov   (x),a
15dc: c6        mov   (x),a
15dd: da 06     movw  $06,ya
15df: db 0c     mov   $0c+x,y
15e1: 0c 6b b0  asl   $b06b
15e4: b0 c7     bcs   $15ad
15e6: b0 c7     bcs   $15af
15e8: b0 b2     bcs   $159c
15ea: c7 b4     mov   ($b4+x),a
15ec: b0 c7     bcs   $15b5
15ee: ad 30     cmp   y,#$30
15f0: ab 00     inc   $00
15f2: da 06     movw  $06,ya
15f4: db 08     mov   $08+x,y
15f6: 0c 6b ac  asl   $ac6b
15f9: ac c7 ac  inc   $acc7
15fc: c7 ac     mov   ($ac+x),a
15fe: ae        pop   a
15ff: c7 ab     mov   ($ab+x),a
1601: a8 c7     sbc   a,#$c7
1603: a8 30     sbc   a,#$30
1605: a4 da     sbc   a,$da
1607: 03 db 14  bbs0  $db,$161e
160a: 0c 69 b8  asl   $b869
160d: c7 bc     mov   ($bc+x),a
160f: b3 c7 b8  bbc5  $c7,$15ca
1612: c7 b7     mov   ($b7+x),a
1614: c7 bc     mov   ($bc+x),a
1616: c7 b4     mov   ($b4+x),a
1618: b7 c7     sbc   a,($c7)+y
161a: bc        inc   a
161b: c7 da     mov   ($da+x),a
161d: 06        or    a,(x)
161e: db 0c     mov   $0c+x,y
1620: 0c 6b b0  asl   $b06b
1623: b0 c7     bcs   $15ec
1625: b0 c7     bcs   $15ee
1627: b0 b2     bcs   $15db
1629: 24 b4     and   a,$b4
162b: c6        mov   (x),a
162c: c6        mov   (x),a
162d: 00        nop
162e: da 06     movw  $06,ya
1630: db 08     mov   $08+x,y
1632: 0c 6b ac  asl   $ac6b
1635: ac c7 ac  inc   $acc7
1638: c7 ac     mov   ($ac+x),a
163a: ae        pop   a
163b: 24 ab     and   a,$ab
163d: c6        mov   (x),a
163e: c6        mov   (x),a
163f: da 0e     movw  $0e,ya
1641: db 0a     mov   $0a+x,y
1643: 24 6d     and   a,$6d
1645: 88 8f     adc   a,#$8f
1647: 18 94 24  or    $24,#$94
164a: 93 8c 18  bbc4  $8c,$1665
164d: 87 da     adc   a,($da+x)
164f: 06        or    a,(x)
1650: db 0a     mov   $0a+x,y
1652: 0c 6b b4  asl   $b46b
1655: b4 c7     sbc   a,$c7+x
1657: b4 c7     sbc   a,$c7+x
1659: b0 b4     bcs   $160f
165b: c7 18     mov   ($18+x),a
165d: b7 c7     sbc   a,($c7)+y
165f: da 0f     movw  $0f,ya
1661: a3 c7 00  bbs5  $c7,$1664
1664: da 06     movw  $06,ya
1666: db 0a     mov   $0a+x,y
1668: 0c 6b aa  asl   $aa6b
166b: aa c7 aa  mov1  c,$0ac7,5
166e: c7 aa     mov   ($aa+x),a
1670: aa c7 18  mov1  c,$18c7,0
1673: af        mov   (x)+,a
1674: c7 da     mov   ($da+x),a
1676: 0f        brk
1677: a3 c7 da  bbs5  $c7,$1654
167a: 0e 0c 6d  tset1 $6d0c
167d: 8e        pop   psw
167e: 8e        pop   psw
167f: c7 8e     mov   ($8e+x),a
1681: c7 8e     mov   ($8e+x),a
1683: 90 91     bcc   $1616
1685: 18 93 c7  or    $c7,#$93
1688: 93 c7 da  bbc4  $c7,$1665
168b: 04 db     or    a,$db
168d: 0a 0c 6b  or1   c,$0b0c,3
1690: ad ad     cmp   y,#$ad
1692: c7 ad     mov   ($ad+x),a
1694: c7 ad     mov   ($ad+x),a
1696: ad c7     cmp   y,#$c7
1698: 18 b2 c7  or    $c7,#$b2
169b: c7 c7     mov   ($c7+x),a
169d: da 07     movw  $07,ya
169f: db 0a     mov   $0a+x,y
16a1: 0c 6b b4  asl   $b46b
16a4: b0 c7     bcs   $166d
16a6: 24 ab     and   a,$ab
16a8: 18 ac 0c  or    $0c,#$ac
16ab: ad b5     cmp   y,#$b5
16ad: c7 b5     mov   ($b5+x),a
16af: 30 ad     bmi   $165e
16b1: 00        nop
16b2: da 07     movw  $07,ya
16b4: db 0a     mov   $0a+x,y
16b6: 0c 6b b0  asl   $b06b
16b9: 18 ad 24  or    $24,#$ad
16bc: a8 18     sbc   a,#$18
16be: a8 0c     sbc   a,#$0c
16c0: a9 b0 c7  sbc   ($c7),($b0)
16c3: b0 30     bcs   $16f5
16c5: a9 da 0e  sbc   ($0e),($da)
16c8: db 0a     mov   $0a+x,y
16ca: 24 6d     and   a,$6d
16cc: 8c 0c 92  dec   $920c
16cf: 18 93 98  or    $98,#$93
16d2: 24 91     and   a,$91
16d4: 0c 91 0c  asl   $0c91
16d7: 98 98 91  adc   $91,#$98
16da: c7 da     mov   ($da+x),a
16dc: 07 db     or    a,($db+x)
16de: 0a 10 6b  or1   c,$0b10,3
16e1: af        mov   (x)+,a
16e2: b9        sbc   (x),(y)
16e3: b9        sbc   (x),(y)
16e4: b9        sbc   (x),(y)
16e5: b7 b5     sbc   a,($b5)+y
16e7: 0c b4 b0  asl   $b0b4
16ea: c7 ad     mov   ($ad+x),a
16ec: 30 ab     bmi   $1699
16ee: 00        nop
16ef: da 07     movw  $07,ya
16f1: db 0a     mov   $0a+x,y
16f3: 10 6b     bpl   $1760
16f5: ab b5     inc   $b5
16f7: b5 b5 b4  sbc   a,$b4b5+x
16fa: b2 0c     clr5  $0c
16fc: b0 ad     bcs   $16ab
16fe: c7 a9     mov   ($a9+x),a
1700: 30 a8     bmi   $16aa
1702: da 0e     movw  $0e,ya
1704: db 0a     mov   $0a+x,y
1706: 24 6d     and   a,$6d
1708: 8e        pop   psw
1709: 0c 91 18  asl   $1891
170c: 93 97 24  bbc4  $97,$1733
170f: 93 0c 93  bbc4  $0c,$16a5
1712: 0c 98 98  asl   $9898
1715: 93 c7 da  bbc4  $c7,$16f2
1718: 07 db     or    a,($db+x)
171a: 0a 0c 6b  or1   c,$0b0c,3
171d: af        mov   (x)+,a
171e: b5 c7 b5  sbc   a,$b5c7+x
1721: 10 b5     bpl   $16d8
1723: b4 b2     sbc   a,$b2+x
1725: 30 b0     bmi   $16d7
1727: c7 00     mov   ($00+x),a
1729: da 07     movw  $07,ya
172b: db 0a     mov   $0a+x,y
172d: 0c 6b ab  asl   $ab6b
1730: b2 c7     clr5  $c7
1732: b2 10     clr5  $10
1734: b2 b0     clr5  $b0
1736: af        mov   (x)+,a
1737: 0c ab a8  asl   $a8ab
173a: c7 a8     mov   ($a8+x),a
173c: 30 a4     bmi   $16e2
173e: da 0e     movw  $0e,ya
1740: db 0a     mov   $0a+x,y
1742: 24 6d     and   a,$6d
1744: 93 0c 93  bbc4  $0c,$16da
1747: 10 93     bpl   $16dc
1749: 95 97 18  adc   a,$1897+x
174c: 98 93 30  adc   $30,#$93
174f: 8c 00 f0  dec   $f000
1752: da 04     movw  $04,ya
1754: e2 23     set7  $23
1756: db 0a     mov   $0a+x,y
1758: de 14 15  cbne  $14+x,$1770
175b: 20        clrp
175c: 30 7e     bmi   $17dc
175e: c7 0c     mov   ($0c+x),a
1760: a9 a9 18  sbc   ($18),($a9)
1763: 4e a9 b0  tclr1 $b0a9
1766: 48 7e     eor   a,#$7e
1768: af        mov   (x)+,a
1769: 0c 4e ae  asl   $ae4e
176c: ad 18     cmp   y,#$18
176e: ac b5 60  inc   $60b5
1771: b4 c6     sbc   a,$c6+x
1773: 00        nop
1774: da 04     movw  $04,ya
1776: db 0a     mov   $0a+x,y
1778: de 14 15  cbne  $14+x,$1790
177b: 20        clrp
177c: 30 7e     bmi   $17fc
177e: c7 0c     mov   ($0c+x),a
1780: 9d        mov   x,sp
1781: 9d        mov   x,sp
1782: 18 4e 9d  or    $9d,#$4e
1785: a4 48     sbc   a,$48
1787: 7e a3     cmp   y,$a3
1789: 0c 4e a2  asl   $a24e
178c: a1        tcall 10
178d: 18 a0 a9  or    $a9,#$a0
1790: 60        clrc
1791: a8 c6     sbc   a,#$c6
1793: da 04     movw  $04,ya
1795: db 0a     mov   $0a+x,y
1797: de 14 15  cbne  $14+x,$17af
179a: 20        clrp
179b: 30 7e     bmi   $181b
179d: c7 0c     mov   ($0c+x),a
179f: a4 a4     sbc   a,$a4
17a1: 18 4e a4  or    $a4,#$4e
17a4: ab 48     inc   $48
17a6: 7e aa     cmp   y,$aa
17a8: 0c 4e a9  asl   $a94e
17ab: a8 18     sbc   a,#$18
17ad: a7 b0     sbc   a,($b0+x)
17af: 60        clrc
17b0: af        mov   (x)+,a
17b1: c6        mov   (x),a
17b2: da 04     movw  $04,ya
17b4: db 0a     mov   $0a+x,y
17b6: de 14 15  cbne  $14+x,$17ce
17b9: 20        clrp
17ba: 30 7e     bmi   $183a
17bc: c7 0c     mov   ($0c+x),a
17be: 9f        xcn   a
17bf: 9f        xcn   a
17c0: 18 4e 9f  or    $9f,#$4e
17c3: a6        sbc   a,(x)
17c4: 48 7e     eor   a,#$7e
17c6: a5 0c 4e  sbc   a,$4e0c
17c9: a4 a3     sbc   a,$a3
17cb: 18 a2 ab  or    $ab,#$a2
17ce: 60        clrc
17cf: aa c6 da  mov1  c,$1ac6,6
17d2: 04 db     or    a,$db
17d4: 0a de 14  or1   c,$14de,0
17d7: 1f 30 0c  jmp   ($0c30+x)
17da: 4f 8c     pcall $8c
17dc: 8c 60 8c  dec   $8c60
17df: 18 c6 06  or    $06,#$c6
17e2: 3f 8c 8c  call  $8c8c
17e5: 8c 8c 48  dec   $488c
17e8: 6f        ret
17e9: 8c 18 8c  dec   $8c18
17ec: 60        clrc
17ed: 8d c6     mov   y,#$c6
17ef: da 04     movw  $04,ya
17f1: db 0a     mov   $0a+x,y
17f3: 0c 4f d0  asl   $d04f
17f6: d0 60     bne   $1858
17f8: d0 18     bne   $1812
17fa: c6        mov   (x),a
17fb: 06        or    a,(x)
17fc: 3f d0 d0  call  $d0d0
17ff: d0 d0     bne   $17d1
1801: 48 6f     eor   a,#$6f
1803: d0 18     bne   $181d
1805: d0 60     bne   $1867
1807: d0 c7     bne   $17d0
1809: f0 18     beq   $1823
180b: c7 00     mov   ($00+x),a
180d: da 01     movw  $01,ya
180f: e2 19     set7  $19
1811: db 0d     mov   $0d+x,y
1813: de 14 1f  cbne  $14+x,$1835
1816: 30 06     bmi   $181e
1818: 4e b0 b2  tclr1 $b2b0
181b: b0 c6     bcs   $17e3
181d: b0 b2     bcs   $17d1
181f: b0 c6     bcs   $17e7
1821: 0c 3e b0  asl   $b03e
1824: 18 5e bc  or    $bc,#$5e
1827: 0c bc 0c  asl   $0cbc
182a: 5c        lsr   a
182b: bb 0c     inc   $0c+x
182d: 2c b2 b2  rol   $b2b2
1830: b2 30     clr5  $30
1832: 6d        push  y
1833: b2 00     clr5  $00
1835: da 01     movw  $01,ya
1837: db 08     mov   $08+x,y
1839: de 14 1e  cbne  $14+x,$185a
183c: 30 30     bmi   $186e
183e: 7d        mov   a,x
183f: c7 0c     mov   ($0c+x),a
1841: a7 24     sbc   a,($24+x)
1843: ac 30 c7  inc   $c730
1846: 0c 7c ae  asl   $ae7c
1849: 24 af     and   a,$af
184b: da 01     movw  $01,ya
184d: db 0a     mov   $0a+x,y
184f: de 14 1d  cbne  $14+x,$186f
1852: 30 18     bmi   $186c
1854: 7d        mov   a,x
1855: c7 c7     mov   ($c7+x),a
1857: 0c a4 24  asl   $24a4
185a: a7 18     sbc   a,($18+x)
185c: c7 c7     mov   ($c7+x),a
185e: 0c 7c aa  asl   $aa7c
1861: 24 ab     and   a,$ab
1863: da 01     movw  $01,ya
1865: db 05     mov   $05+x,y
1867: de 14 1e  cbne  $14+x,$1888
186a: 30 18     bmi   $1884
186c: c7 48     mov   ($48+x),a
186e: 4e 9b 18  tclr1 $189b
1871: c7 48     mov   ($48+x),a
1873: 4d        push  x
1874: 9f        xcn   a
1875: da 01     movw  $01,ya
1877: db 00     mov   $00+x,y
1879: de 14 1c  cbne  $14+x,$1898
187c: 30 60     bmi   $18de
187e: 4f 94     pcall $94
1880: 60        clrc
1881: 4e 93 0c  tclr1 $0c93
1884: 6c b5 0c  ror   $0cb5
1887: 2b b3     rol   $b3
1889: b3 0c 6c  bbc5  $0c,$18f8
188c: b5 0c 2b  sbc   a,$2b0c+x
188f: b3 b3 18  bbc5  $b3,$18aa
1892: 6c b5 0c  ror   $0cb5
1895: 6c b4 0c  ror   $0cb4
1898: 2b b2     rol   $b2
189a: b2 0c     clr5  $0c
189c: 6c b4 0c  ror   $0cb4
189f: 2b b2     rol   $b2
18a1: b2 18     clr5  $18
18a3: 6c b4 0c  ror   $0cb4
18a6: 6d        push  y
18a7: b5 0c 2c  sbc   a,$2c0c+x
18aa: b3 b3 0c  bbc5  $b3,$18b9
18ad: 6d        push  y
18ae: b5 0c 2c  sbc   a,$2c0c+x
18b1: b3 b3 0c  bbc5  $b3,$18c0
18b4: 6d        push  y
18b5: b5 bc 0c  sbc   a,$0cbc+x
18b8: 7c        ror   a
18b9: c6        mov   (x),a
18ba: 54 7c     eor   a,$7c+x
18bc: bb 00     inc   $00+x
18be: 24 3b     and   a,$3b
18c0: ae        pop   a
18c1: ae        pop   a
18c2: 18 6b ae  or    $ae,#$6b
18c5: 24 3b     and   a,$3b
18c7: ad ad     cmp   y,#$ad
18c9: 18 6b ad  or    $ad,#$6b
18cc: 24 3c     and   a,$3c
18ce: ae        pop   a
18cf: ae        pop   a
18d0: 18 6c ae  or    $ae,#$6c
18d3: 60        clrc
18d4: 5d        mov   x,a
18d5: ad 24     cmp   y,#$24
18d7: 3b aa     rol   $aa+x
18d9: aa 18 6b  mov1  c,$0b18,3
18dc: aa 24 3b  mov1  c,$1b24,1
18df: a9 a9 18  sbc   ($18),($a9)
18e2: 6b a9     ror   $a9
18e4: 24 3c     and   a,$3c
18e6: aa aa 18  mov1  c,$18aa,0
18e9: 6c aa 60  ror   $60aa
18ec: 5d        mov   x,a
18ed: a9 f0 da  sbc   ($da),($f0)
18f0: 04 e2     or    a,$e2
18f2: 19        or    (x),(y)
18f3: db 0a     mov   $0a+x,y
18f5: 0c 3b b0  asl   $b03b
18f8: b0 b0     bcs   $18aa
18fa: a6        sbc   a,(x)
18fb: 18 b0 0c  or    $0c,#$b0
18fe: a6        sbc   a,(x)
18ff: af        mov   (x)+,a
1900: c6        mov   (x),a
1901: af        mov   (x)+,a
1902: af        mov   (x)+,a
1903: a4 af     sbc   a,$af
1905: ad ae     cmp   y,#$ae
1907: af        mov   (x)+,a
1908: 00        nop
1909: da 04     movw  $04,ya
190b: db 00     mov   $00+x,y
190d: 0c 3b ad  asl   $ad3b
1910: ad ad     cmp   y,#$ad
1912: c7 18     mov   ($18+x),a
1914: ad 0c     cmp   y,#$0c
1916: c7 ab     mov   ($ab+x),a
1918: c6        mov   (x),a
1919: ab ab     inc   $ab
191b: c7 ab     mov   ($ab+x),a
191d: c7 c7     mov   ($c7+x),a
191f: c7 da     mov   ($da+x),a
1921: 04 db     or    a,$db
1923: 00        nop
1924: 0c 3b a9  asl   $a93b
1927: a9 a9 c7  sbc   ($c7),($a9)
192a: 18 a9 0c  or    $0c,#$a9
192d: c7 a8     mov   ($a8+x),a
192f: c6        mov   (x),a
1930: a8 a8     sbc   a,#$a8
1932: c7 a8     mov   ($a8+x),a
1934: c7 c7     mov   ($c7+x),a
1936: c7 da     mov   ($da+x),a
1938: 08 db     or    a,#$db
193a: 0a 24 3e  or1   c,$1e24,1
193d: 8e        pop   psw
193e: 0c 95 24  asl   $2495
1941: 9a 0c     subw  ya,$0c
1943: 95 24 8c  adc   a,$8c24+x
1946: 0c 93 98  asl   $9893
1949: 98 93 c6  adc   $c6,#$93
194c: da 0c     movw  $0c,ya
194e: db 00     mov   $00+x,y
1950: 0c 6f a6  asl   $a66f
1953: c6        mov   (x),a
1954: ad c6     cmp   y,#$c6
1956: c6        mov   (x),a
1957: ad a3     cmp   y,#$a3
1959: a3 a6 c6  bbs5  $a6,$1922
195c: ad c6     cmp   y,#$c6
195e: c6        mov   (x),a
195f: ad a3     cmp   y,#$a3
1961: a3 da 0a  bbs5  $da,$196e
1964: db 14     mov   $14+x,y
1966: 0c 6b c7  asl   $c76b
1969: c6        mov   (x),a
196a: d2 c6     clr6  $c6
196c: c6        mov   (x),a
196d: c6        mov   (x),a
196e: 0c 69 d1  asl   $d169
1971: d1        tcall 13
1972: 0c 6b c7  asl   $c76b
1975: c6        mov   (x),a
1976: d2 c6     clr6  $c6
1978: c6        mov   (x),a
1979: c6        mov   (x),a
197a: 0c 69 d1  asl   $d169
197d: d1        tcall 13
197e: 0c b0 b0  asl   $b0b0
1981: b0 a6     bcs   $1929
1983: 18 b0 0c  or    $0c,#$b0
1986: a6        sbc   a,(x)
1987: af        mov   (x)+,a
1988: c6        mov   (x),a
1989: af        mov   (x)+,a
198a: af        mov   (x)+,a
198b: af        mov   (x)+,a
198c: af        mov   (x)+,a
198d: c7 c7     mov   ($c7+x),a
198f: c7 00     mov   ($00+x),a
1991: 0c ad ad  asl   $adad
1994: ad c7     cmp   y,#$c7
1996: 18 ad 0c  or    $0c,#$ad
1999: c7 ab     mov   ($ab+x),a
199b: c6        mov   (x),a
199c: ab ab     inc   $ab
199e: ab ab     inc   $ab
19a0: c7 c7     mov   ($c7+x),a
19a2: c7 0c     mov   ($0c+x),a
19a4: a9 a9 a9  sbc   ($a9),($a9)
19a7: c7 18     mov   ($18+x),a
19a9: a9 0c c7  sbc   ($c7),($0c)
19ac: a8 c6     sbc   a,#$c6
19ae: a8 a8     sbc   a,#$a8
19b0: a8 a8     sbc   a,#$a8
19b2: c7 c7     mov   ($c7+x),a
19b4: c7 24     mov   ($24+x),a
19b6: 8e        pop   psw
19b7: 0c 95 24  asl   $2495
19ba: 9a 0c     subw  ya,$0c
19bc: 95 8c 8c  adc   a,$8c8c+x
19bf: 8c 8c 8c  dec   $8c8c
19c2: c6        mov   (x),a
19c3: c6        mov   (x),a
19c4: 93 0c a6  bbc4  $0c,$196d
19c7: c6        mov   (x),a
19c8: ad c6     cmp   y,#$c6
19ca: c6        mov   (x),a
19cb: ad a3     cmp   y,#$a3
19cd: a3 a6 c6  bbs5  $a6,$1996
19d0: ad ad     cmp   y,#$ad
19d2: ad c6     cmp   y,#$c6
19d4: b0 c6     bcs   $199c
19d6: 0c c7 c6  asl   $c6c7
19d9: d2 c6     clr6  $c6
19db: c6        mov   (x),a
19dc: c6        mov   (x),a
19dd: 0c 69 d1  asl   $d169
19e0: d1        tcall 13
19e1: 0c 6b c7  asl   $c76b
19e4: d1        tcall 13
19e5: 0c 69 d1  asl   $d169
19e8: 0c 6b d1  asl   $d16b
19eb: 30 6c     bmi   $1a59
19ed: d1        tcall 13
19ee: f0 da     beq   $19ca
19f0: 04 e2     or    a,$e2
19f2: 18 db 05  or    $05,#$db
19f5: de 00 00  cbne  $00+x,$19f8
19f8: 00        nop
19f9: 60        clrc
19fa: 49 c6 48  eor   ($48),($c6)
19fd: c6        mov   (x),a
19fe: 0c c7 ac  asl   $acc7
1a01: 00        nop
1a02: da 04     movw  $04,ya
1a04: db 0a     mov   $0a+x,y
1a06: de 00 00  cbne  $00+x,$1a09
1a09: 00        nop
1a0a: 60        clrc
1a0b: 49 c6 48  eor   ($48),($c6)
1a0e: c6        mov   (x),a
1a0f: 0c c7 a7  asl   $a7c7
1a12: da 04     movw  $04,ya
1a14: db 0f     mov   $0f+x,y
1a16: de 00 00  cbne  $00+x,$1a19
1a19: 00        nop
1a1a: 60        clrc
1a1b: 49 c6 48  eor   ($48),($c6)
1a1e: c6        mov   (x),a
1a1f: 0c c7 a1  asl   $a1c7
1a22: 60        clrc
1a23: ab 48     inc   $48
1a25: c6        mov   (x),a
1a26: 0c c7 ab  asl   $abc7
1a29: 00        nop
1a2a: 60        clrc
1a2b: a6        sbc   a,(x)
1a2c: 48 c6     eor   a,#$c6
1a2e: 0c c7 a6  asl   $a6c7
1a31: 60        clrc
1a32: a0        ei
1a33: 48 c6     eor   a,#$c6
1a35: 0c c7 a0  asl   $a0c7
1a38: 60        clrc
1a39: ac 48 c6  inc   $c648
1a3c: 0c c7 ac  asl   $acc7
1a3f: 00        nop
1a40: 60        clrc
1a41: a7 48     sbc   a,($48+x)
1a43: c6        mov   (x),a
1a44: 0c c7 a7  asl   $a7c7
1a47: 60        clrc
1a48: a1        tcall 10
1a49: 48 c6     eor   a,#$c6
1a4b: 0c c7 a1  asl   $a1c7
1a4e: da 0e     movw  $0e,ya
1a50: e7 ff     mov   a,($ff+x)
1a52: db 0a     mov   $0a+x,y
1a54: 06        or    a,(x)
1a55: 6f        ret
1a56: c7 84     mov   ($84+x),a
1a58: 84 84     adc   a,$84
1a5a: 84 c6     adc   a,$c6
1a5c: 84 84     adc   a,$84
1a5e: c6        mov   (x),a
1a5f: 84 84     adc   a,$84
1a61: c6        mov   (x),a
1a62: 84 c6     adc   a,$c6
1a64: 0c 9a dd  asl   $dd9a
1a67: 06        or    a,(x)
1a68: 04 9c     or    a,$9c
1a6a: 06        or    a,(x)
1a6b: c7 84     mov   ($84+x),a
1a6d: 84 84     adc   a,$84
1a6f: 84 c6     adc   a,$c6
1a71: 84 84     adc   a,$84
1a73: c6        mov   (x),a
1a74: 84 84     adc   a,$84
1a76: c6        mov   (x),a
1a77: 84 c6     adc   a,$c6
1a79: 0c 95 dd  asl   $dd95
1a7c: 06        or    a,(x)
1a7d: 04 97     or    a,$97
1a7f: da 05     movw  $05,ya
1a81: db 14     mov   $14+x,y
1a83: de 00 00  cbne  $00+x,$1a86
1a86: 00        nop
1a87: e9 8b 1f  mov   x,$1f8b
1a8a: 20        clrp
1a8b: 06        or    a,(x)
1a8c: 49 d1 06  eor   ($06),($d1)
1a8f: 49 d1 06  eor   ($06),($d1)
1a92: 4b d1     lsr   $d1
1a94: 06        or    a,(x)
1a95: 49 d1 06  eor   ($06),($d1)
1a98: 4c d1 06  lsr   $06d1
1a9b: 49 d1 06  eor   ($06),($d1)
1a9e: 4b d1     lsr   $d1
1aa0: 06        or    a,(x)
1aa1: 49 d1 06  eor   ($06),($d1)
1aa4: 49 d1 06  eor   ($06),($d1)
1aa7: 49 d1 06  eor   ($06),($d1)
1aaa: 4b d1     lsr   $d1
1aac: 06        or    a,(x)
1aad: 49 d1 06  eor   ($06),($d1)
1ab0: 4c d1 06  lsr   $06d1
1ab3: 49 d1 06  eor   ($06),($d1)
1ab6: 4b d1     lsr   $d1
1ab8: 06        or    a,(x)
1ab9: 49 d1 00  eor   ($00),($d1)
1abc: da 05     movw  $05,ya
1abe: e7 ff     mov   a,($ff+x)
1ac0: db 0a     mov   $0a+x,y
1ac2: de 00 00  cbne  $00+x,$1ac5
1ac5: 00        nop
1ac6: 18 4f d0  or    $d0,#$4f
1ac9: d8 0c     mov   $0c,x
1acb: c7 d0     mov   ($d0+x),a
1acd: 18 d8 d0  or    $d0,#$d8
1ad0: d8 0c     mov   $0c,x
1ad2: c7 d0     mov   ($d0+x),a
1ad4: 18 d8 f0  or    $f0,#$d8
1ad7: da 03     movw  $03,ya
1ad9: e2 1c     set7  $1c
1adb: db 0a     mov   $0a+x,y
1add: 12 5d     clr0  $5d
1adf: a8 06     sbc   a,#$06
1ae1: a8 0c     sbc   a,#$0c
1ae3: a8 9f     sbc   a,#$9f
1ae5: 18 a1 9f  or    $9f,#$a1
1ae8: 0c a8 a8  asl   $a8a8
1aeb: a8 da     sbc   a,#$da
1aed: 00        nop
1aee: db 00     mov   $00+x,y
1af0: 0c b7 0c  asl   $0cb7
1af3: b9        sbc   (x),(y)
1af4: c6        mov   (x),a
1af5: b7 c7     sbc   a,($c7)+y
1af7: da 03     movw  $03,ya
1af9: db 0a     mov   $0a+x,y
1afb: 12 a9     clr0  $a9
1afd: 06        or    a,(x)
1afe: a9 0c a9  sbc   ($a9),($0c)
1b01: a1        tcall 10
1b02: 18 a3 a1  or    $a1,#$a3
1b05: 0c a9 a9  asl   $a9a9
1b08: a9 da 00  sbc   ($00),($da)
1b0b: db 00     mov   $00+x,y
1b0d: 0c b9 0c  asl   $0cb9
1b10: bb c6     inc   $c6+x
1b12: b9        sbc   (x),(y)
1b13: c7 00     mov   ($00+x),a
1b15: da 03     movw  $03,ya
1b17: db 0a     mov   $0a+x,y
1b19: 12 5d     clr0  $5d
1b1b: a4 06     sbc   a,$06
1b1d: a4 0c     sbc   a,$0c
1b1f: a4 9c     sbc   a,$9c
1b21: 18 9d 9c  or    $9c,#$9d
1b24: 0c a4 a4  asl   $a4a4
1b27: a4 da     sbc   a,$da
1b29: 00        nop
1b2a: db 00     mov   $00+x,y
1b2c: 0c b4 0c  asl   $0cb4
1b2f: b5 c6 b4  sbc   a,$b4c6+x
1b32: c7 da     mov   ($da+x),a
1b34: 03 db 0a  bbs0  $db,$1b41
1b37: 12 a6     clr0  $a6
1b39: 06        or    a,(x)
1b3a: a6        sbc   a,(x)
1b3b: 0c a6 9d  asl   $9da6
1b3e: 18 9f 9d  or    $9d,#$9f
1b41: 0c a6 a6  asl   $a6a6
1b44: a6        sbc   a,(x)
1b45: da 00     movw  $00,ya
1b47: db 00     mov   $00+x,y
1b49: 0c b5 0c  asl   $0cb5
1b4c: b7 c6     sbc   a,($c6)+y
1b4e: b5 c7 da  sbc   a,$dac7+x
1b51: 04 db     or    a,$db
1b53: 0a 24 6d  or1   c,$0d24,3
1b56: 8c 0c 93  dec   $930c
1b59: 18 98 93  or    $93,#$98
1b5c: 0c 8c 8c  asl   $8c8c
1b5f: 8c c7 0c  dec   $0cc7
1b62: c7 c7     mov   ($c7+x),a
1b64: 8c c6 24  dec   $24c6
1b67: 8e        pop   psw
1b68: 0c 95 18  asl   $1895
1b6b: 9a 95     subw  ya,$95
1b6d: 0c 8e 8e  asl   $8e8e
1b70: 8e        pop   psw
1b71: c7 0c     mov   ($0c+x),a
1b73: c7 c7     mov   ($c7+x),a
1b75: 8e        pop   psw
1b76: c6        mov   (x),a
1b77: da 04     movw  $04,ya
1b79: db 14     mov   $14+x,y
1b7b: 06        or    a,(x)
1b7c: 6b d1     ror   $d1
1b7e: 06        or    a,(x)
1b7f: 69 d1 d1  cmp   ($d1),($d1)
1b82: d1        tcall 13
1b83: 0c d1 d1  asl   $d1d1
1b86: 18 d1 d1  or    $d1,#$d1
1b89: 0c 6b d1  asl   $d16b
1b8c: d1        tcall 13
1b8d: d1        tcall 13
1b8e: c7 30     mov   ($30+x),a
1b90: c7 06     mov   ($06+x),a
1b92: 6b d1     ror   $d1
1b94: 06        or    a,(x)
1b95: 69 d1 d1  cmp   ($d1),($d1)
1b98: d1        tcall 13
1b99: 0c d1 d1  asl   $d1d1
1b9c: 18 d1 d1  or    $d1,#$d1
1b9f: 0c 6b d1  asl   $d16b
1ba2: d1        tcall 13
1ba3: d1        tcall 13
1ba4: c7 30     mov   ($30+x),a
1ba6: c7 da     mov   ($da+x),a
1ba8: 03 e2 1c  bbs0  $e2,$1bc7
1bab: db 0a     mov   $0a+x,y
1bad: 12 5d     clr0  $5d
1baf: a8 06     sbc   a,#$06
1bb1: a8 0c     sbc   a,#$0c
1bb3: a8 9f     sbc   a,#$9f
1bb5: 18 a1 9f  or    $9f,#$a1
1bb8: da 01     movw  $01,ya
1bba: 0c a8 a7  asl   $a7a8
1bbd: a6        sbc   a,(x)
1bbe: a5 30 a4  sbc   a,$a430
1bc1: 00        nop
1bc2: da 03     movw  $03,ya
1bc4: db 0a     mov   $0a+x,y
1bc6: 12 5d     clr0  $5d
1bc8: a4 06     sbc   a,$06
1bca: a4 0c     sbc   a,$0c
1bcc: a4 9c     sbc   a,$9c
1bce: 18 9d 9c  or    $9c,#$9d
1bd1: da 01     movw  $01,ya
1bd3: 0c a2 a1  asl   $a1a2
1bd6: a0        ei
1bd7: 9f        xcn   a
1bd8: 30 9e     bmi   $1b78
1bda: da 04     movw  $04,ya
1bdc: db 0a     mov   $0a+x,y
1bde: 24 6d     and   a,$6d
1be0: 8c 0c 93  dec   $930c
1be3: 18 98 93  or    $93,#$98
1be6: 0c 8c 8b  asl   $8b8c
1be9: 8a 89 30  eor1  c,$1089,1
1bec: 88 da     adc   a,#$da
1bee: 04 db     or    a,$db
1bf0: 14 06     or    a,$06+x
1bf2: 6b d1     ror   $d1
1bf4: 06        or    a,(x)
1bf5: 69 d1 d1  cmp   ($d1),($d1)
1bf8: d1        tcall 13
1bf9: 0c d1 d1  asl   $d1d1
1bfc: 18 d1 d1  or    $d1,#$d1
1bff: 48 c7     eor   a,#$c7
1c01: d4 da     mov   $da+x,a
1c03: 08 e2     or    a,#$e2
1c05: 1c        asl   a
1c06: db 0a     mov   $0a+x,y
1c08: 18 4f 8c  or    $8c,#$4f
1c0b: 98 89 95  adc   $95,#$89
1c0e: 8e        pop   psw
1c0f: 9a 87     subw  ya,$87
1c11: 93 8c 98  bbc4  $8c,$1bac
1c14: 89 95 60  adc   ($60),($95)
1c17: c7 00     mov   ($00+x),a
1c19: da 03     movw  $03,ya
1c1b: db 0a     mov   $0a+x,y
1c1d: 08 6f     or    a,#$6f
1c1f: c7 c7     mov   ($c7+x),a
1c21: a4 af     sbc   a,$af
1c23: a4 af     sbc   a,$af
1c25: 18 c7 ad  or    $ad,#$c7
1c28: 08 c7     or    a,#$c7
1c2a: c7 a6     mov   ($a6+x),a
1c2c: b0 a6     bcs   $1bd4
1c2e: b0 18     bcs   $1c48
1c30: c7 af     mov   ($af+x),a
1c32: 08 c7     or    a,#$c7
1c34: c7 a4     mov   ($a4+x),a
1c36: af        mov   (x)+,a
1c37: a4 af     sbc   a,$af
1c39: 18 c7 ad  or    $ad,#$c7
1c3c: 08 a6     or    a,#$a6
1c3e: c7 a8     mov   ($a8+x),a
1c40: a9 c7 ab  sbc   ($ab),($c7)
1c43: da 00     movw  $00,ya
1c45: 08 c7     or    a,#$c7
1c47: c7 04     mov   ($04+x),a
1c49: b7 b9     sbc   a,($b9)+y
1c4b: 0c b7 c7  asl   $c7b7
1c4e: da 03     movw  $03,ya
1c50: db 00     mov   $00+x,y
1c52: 08 6f     or    a,#$6f
1c54: c7 c7     mov   ($c7+x),a
1c56: c7 ab     mov   ($ab+x),a
1c58: c7 ab     mov   ($ab+x),a
1c5a: 18 c7 a8  or    $a8,#$c7
1c5d: 08 c7     or    a,#$c7
1c5f: c7 c7     mov   ($c7+x),a
1c61: ad c7     cmp   y,#$c7
1c63: ad 18     cmp   y,#$18
1c65: c7 ab     mov   ($ab+x),a
1c67: 08 c7     or    a,#$c7
1c69: c7 c7     mov   ($c7+x),a
1c6b: ab c7     inc   $c7
1c6d: ab 18     inc   $18
1c6f: c7 a8     mov   ($a8+x),a
1c71: 60        clrc
1c72: c7 da     mov   ($da+x),a
1c74: 03 db 14  bbs0  $db,$1c8b
1c77: 08 6f     or    a,#$6f
1c79: c7 c7     mov   ($c7+x),a
1c7b: c7 a8     mov   ($a8+x),a
1c7d: c7 a8     mov   ($a8+x),a
1c7f: 18 c7 a4  or    $a4,#$c7
1c82: 08 c7     or    a,#$c7
1c84: c7 c7     mov   ($c7+x),a
1c86: a9 c7 a9  sbc   ($a9),($c7)
1c89: 18 c7 a6  or    $a6,#$c7
1c8c: 08 c7     or    a,#$c7
1c8e: c7 c7     mov   ($c7+x),a
1c90: a8 c7     sbc   a,#$c7
1c92: a8 18     sbc   a,#$18
1c94: c7 a4     mov   ($a4+x),a
1c96: 60        clrc
1c97: c7 08     mov   ($08+x),a
1c99: d1        tcall 13
1c9a: c7 d1     mov   ($d1+x),a
1c9c: 18 d2 00  or    $00,#$d2
1c9f: da 08     movw  $08,ya
1ca1: db 0a     mov   $0a+x,y
1ca3: 18 4f 8c  or    $8c,#$4f
1ca6: 98 89 95  adc   $95,#$89
1ca9: 8e        pop   psw
1caa: 9a 87     subw  ya,$87
1cac: 93 8c 98  bbc4  $8c,$1c47
1caf: 89 95 0c  adc   ($0c),($95)
1cb2: 8e        pop   psw
1cb3: c7 18     mov   ($18+x),a
1cb5: c7 30     mov   ($30+x),a
1cb7: 8d 00     mov   y,#$00
1cb9: f0 da     beq   $1c95
1cbb: 03 e2 1c  bbs0  $e2,$1cda
1cbe: db 0a     mov   $0a+x,y
1cc0: 08 6f     or    a,#$6f
1cc2: c7 c7     mov   ($c7+x),a
1cc4: a4 af     sbc   a,$af
1cc6: a4 af     sbc   a,$af
1cc8: 18 c7 ad  or    $ad,#$c7
1ccb: 08 c7     or    a,#$c7
1ccd: c7 a6     mov   ($a6+x),a
1ccf: b0 a6     bcs   $1c77
1cd1: b0 18     bcs   $1ceb
1cd3: c7 af     mov   ($af+x),a
1cd5: 08 c7     or    a,#$c7
1cd7: c7 a4     mov   ($a4+x),a
1cd9: af        mov   (x)+,a
1cda: a4 af     sbc   a,$af
1cdc: 18 c7 ad  or    $ad,#$c7
1cdf: 08 b0     or    a,#$b0
1ce1: c7 ad     mov   ($ad+x),a
1ce3: c7 a9     mov   ($a9+x),a
1ce5: c7 18     mov   ($18+x),a
1ce7: a8 a6     sbc   a,#$a6
1ce9: da 03     movw  $03,ya
1ceb: db 00     mov   $00+x,y
1ced: 08 6f     or    a,#$6f
1cef: c7 c7     mov   ($c7+x),a
1cf1: c7 ab     mov   ($ab+x),a
1cf3: c7 ab     mov   ($ab+x),a
1cf5: 18 c7 a8  or    $a8,#$c7
1cf8: 08 c7     or    a,#$c7
1cfa: c7 c7     mov   ($c7+x),a
1cfc: ad c7     cmp   y,#$c7
1cfe: ad 18     cmp   y,#$18
1d00: c7 ab     mov   ($ab+x),a
1d02: 08 c7     or    a,#$c7
1d04: c7 c7     mov   ($c7+x),a
1d06: ab c7     inc   $c7
1d08: ab 18     inc   $18
1d0a: c7 a8     mov   ($a8+x),a
1d0c: 18 ad c7  or    $c7,#$ad
1d0f: 30 a3     bmi   $1cb4
1d11: da 03     movw  $03,ya
1d13: db 14     mov   $14+x,y
1d15: 08 6f     or    a,#$6f
1d17: c7 c7     mov   ($c7+x),a
1d19: c7 a8     mov   ($a8+x),a
1d1b: c7 a8     mov   ($a8+x),a
1d1d: 18 c7 a4  or    $a4,#$c7
1d20: 08 c7     or    a,#$c7
1d22: c7 c7     mov   ($c7+x),a
1d24: a9 c7 a9  sbc   ($a9),($c7)
1d27: 18 c7 a6  or    $a6,#$c7
1d2a: 08 c7     or    a,#$c7
1d2c: c7 c7     mov   ($c7+x),a
1d2e: a8 c7     sbc   a,#$c7
1d30: a8 18     sbc   a,#$18
1d32: c7 a4     mov   ($a4+x),a
1d34: 18 a9 c7  or    $c7,#$a9
1d37: 30 9d     bmi   $1cd6
1d39: da 02     movw  $02,ya
1d3b: db 14     mov   $14+x,y
1d3d: 08 6c     or    a,#$6c
1d3f: d1        tcall 13
1d40: c7 d1     mov   ($d1+x),a
1d42: 18 d2 e9  or    $e9,#$d2
1d45: 98 21 05  adc   $05,#$21
1d48: 18 d1 08  or    $08,#$d1
1d4b: d1        tcall 13
1d4c: d1        tcall 13
1d4d: d1        tcall 13
1d4e: 18 d2 d2  or    $d2,#$d2
1d51: 00        nop
1d52: da 02     movw  $02,ya
1d54: db 05     mov   $05+x,y
1d56: de 00 00  cbne  $00+x,$1d59
1d59: 00        nop
1d5a: 06        or    a,(x)
1d5b: 6b b7     ror   $b7
1d5d: c6        mov   (x),a
1d5e: c6        mov   (x),a
1d5f: c6        mov   (x),a
1d60: ab c6     inc   $c6
1d62: c6        mov   (x),a
1d63: c6        mov   (x),a
1d64: 60        clrc
1d65: c6        mov   (x),a
1d66: 30 c3     bmi   $1d2b
1d68: 06        or    a,(x)
1d69: b6 c6 c6  sbc   a,$c6c6+y
1d6c: c6        mov   (x),a
1d6d: aa c6 c6  mov1  c,$06c6,6
1d70: c6        mov   (x),a
1d71: 60        clrc
1d72: c6        mov   (x),a
1d73: 30 c2     bmi   $1d37
1d75: 00        nop
1d76: da 02     movw  $02,ya
1d78: db 00     mov   $00+x,y
1d7a: de 00 00  cbne  $00+x,$1d7d
1d7d: 00        nop
1d7e: 06        or    a,(x)
1d7f: 6b c6     ror   $c6
1d81: b1        tcall 11
1d82: c6        mov   (x),a
1d83: c6        mov   (x),a
1d84: c6        mov   (x),a
1d85: b1        tcall 11
1d86: c6        mov   (x),a
1d87: c6        mov   (x),a
1d88: 60        clrc
1d89: c6        mov   (x),a
1d8a: 30 c6     bmi   $1d52
1d8c: 06        or    a,(x)
1d8d: c6        mov   (x),a
1d8e: b0 c6     bcs   $1d56
1d90: c6        mov   (x),a
1d91: c6        mov   (x),a
1d92: b0 c6     bcs   $1d5a
1d94: c6        mov   (x),a
1d95: 60        clrc
1d96: c6        mov   (x),a
1d97: 30 c6     bmi   $1d5f
1d99: da 02     movw  $02,ya
1d9b: db 14     mov   $14+x,y
1d9d: de 00 00  cbne  $00+x,$1da0
1da0: 00        nop
1da1: 06        or    a,(x)
1da2: 6d        push  y
1da3: c6        mov   (x),a
1da4: c6        mov   (x),a
1da5: ab c6     inc   $c6
1da7: c6        mov   (x),a
1da8: c6        mov   (x),a
1da9: b7 c6     sbc   a,($c6)+y
1dab: 60        clrc
1dac: c6        mov   (x),a
1dad: 30 c6     bmi   $1d75
1daf: 06        or    a,(x)
1db0: c6        mov   (x),a
1db1: c6        mov   (x),a
1db2: aa c6 c6  mov1  c,$06c6,6
1db5: c6        mov   (x),a
1db6: b6 c6 60  sbc   a,$60c6+y
1db9: c6        mov   (x),a
1dba: 30 c6     bmi   $1d82
1dbc: da 02     movw  $02,ya
1dbe: db 0f     mov   $0f+x,y
1dc0: de 00 00  cbne  $00+x,$1dc3
1dc3: 00        nop
1dc4: 06        or    a,(x)
1dc5: 6d        push  y
1dc6: c6        mov   (x),a
1dc7: c6        mov   (x),a
1dc8: c6        mov   (x),a
1dc9: a5 c6 c6  sbc   a,$c6c6
1dcc: c6        mov   (x),a
1dcd: c6        mov   (x),a
1dce: 5a c6     cmpw  ya,$c6
1dd0: 36 b7 06  and   a,$06b7+y
1dd3: c6        mov   (x),a
1dd4: c6        mov   (x),a
1dd5: c6        mov   (x),a
1dd6: a4 c6     sbc   a,$c6
1dd8: c6        mov   (x),a
1dd9: c6        mov   (x),a
1dda: c6        mov   (x),a
1ddb: 5a c6     cmpw  ya,$c6
1ddd: 36 b6 06  and   a,$06b6+y
1de0: b5 c6 c6  sbc   a,$c6c6+x
1de3: c6        mov   (x),a
1de4: a9 c6 c6  sbc   ($c6),($c6)
1de7: c6        mov   (x),a
1de8: 60        clrc
1de9: c6        mov   (x),a
1dea: 30 c6     bmi   $1db2
1dec: 06        or    a,(x)
1ded: b4 c6     sbc   a,$c6+x
1def: c6        mov   (x),a
1df0: c6        mov   (x),a
1df1: a8 c6     sbc   a,#$c6
1df3: c6        mov   (x),a
1df4: c6        mov   (x),a
1df5: 60        clrc
1df6: c6        mov   (x),a
1df7: 30 c6     bmi   $1dbf
1df9: 00        nop
1dfa: 06        or    a,(x)
1dfb: c6        mov   (x),a
1dfc: af        mov   (x)+,a
1dfd: c6        mov   (x),a
1dfe: c6        mov   (x),a
1dff: c6        mov   (x),a
1e00: af        mov   (x)+,a
1e01: c6        mov   (x),a
1e02: c6        mov   (x),a
1e03: 60        clrc
1e04: c6        mov   (x),a
1e05: 30 c6     bmi   $1dcd
1e07: 06        or    a,(x)
1e08: c6        mov   (x),a
1e09: ae        pop   a
1e0a: c6        mov   (x),a
1e0b: c6        mov   (x),a
1e0c: c6        mov   (x),a
1e0d: ae        pop   a
1e0e: c6        mov   (x),a
1e0f: c6        mov   (x),a
1e10: 60        clrc
1e11: c6        mov   (x),a
1e12: 30 c6     bmi   $1dda
1e14: 06        or    a,(x)
1e15: c6        mov   (x),a
1e16: c6        mov   (x),a
1e17: a9 c6 c6  sbc   ($c6),($c6)
1e1a: c6        mov   (x),a
1e1b: b5 c6 60  sbc   a,$60c6+x
1e1e: c6        mov   (x),a
1e1f: 30 c6     bmi   $1de7
1e21: 06        or    a,(x)
1e22: c6        mov   (x),a
1e23: c6        mov   (x),a
1e24: a8 c6     sbc   a,#$c6
1e26: c6        mov   (x),a
1e27: c6        mov   (x),a
1e28: b4 c6     sbc   a,$c6+x
1e2a: 60        clrc
1e2b: c6        mov   (x),a
1e2c: 30 c6     bmi   $1df4
1e2e: 06        or    a,(x)
1e2f: c6        mov   (x),a
1e30: c6        mov   (x),a
1e31: c6        mov   (x),a
1e32: a3 c6 c6  bbs5  $c6,$1dfb
1e35: c6        mov   (x),a
1e36: c6        mov   (x),a
1e37: 5a c6     cmpw  ya,$c6
1e39: 36 c6 06  and   a,$06c6+y
1e3c: c6        mov   (x),a
1e3d: c6        mov   (x),a
1e3e: c6        mov   (x),a
1e3f: a2 c6     set5  $c6
1e41: c6        mov   (x),a
1e42: c6        mov   (x),a
1e43: c6        mov   (x),a
1e44: 5a c6     cmpw  ya,$c6
1e46: 36 c6 f0  and   a,$f0c6+y
1e49: da 0e     movw  $0e,ya
1e4b: e2 23     set7  $23
1e4d: db 0a     mov   $0a+x,y
1e4f: 30 4e     bmi   $1e9f
1e51: 8e        pop   psw
1e52: 18 c6 0c  or    $0c,#$c6
1e55: 8e        pop   psw
1e56: 8e        pop   psw
1e57: 60        clrc
1e58: c7 30     mov   ($30+x),a
1e5a: 91        tcall 9
1e5b: 18 c6 0c  or    $0c,#$c6
1e5e: 91        tcall 9
1e5f: 91        tcall 9
1e60: 60        clrc
1e61: c7 00     mov   ($00+x),a
1e63: da 07     movw  $07,ya
1e65: db 0a     mov   $0a+x,y
1e67: 30 4e     bmi   $1eb7
1e69: 8e        pop   psw
1e6a: 18 c6 0c  or    $0c,#$c6
1e6d: 8e        pop   psw
1e6e: 8e        pop   psw
1e6f: 60        clrc
1e70: c7 30     mov   ($30+x),a
1e72: 91        tcall 9
1e73: 18 c6 0c  or    $0c,#$c6
1e76: 91        tcall 9
1e77: 91        tcall 9
1e78: 60        clrc
1e79: c7 00     mov   ($00+x),a
1e7b: 30 4e     bmi   $1ecb
1e7d: 94 18     adc   a,$18+x
1e7f: c6        mov   (x),a
1e80: 0c 94 94  asl   $9494
1e83: 60        clrc
1e84: c7 30     mov   ($30+x),a
1e86: 8d 18     mov   y,#$18
1e88: c6        mov   (x),a
1e89: 0c 8d 8d  asl   $8d8d
1e8c: 60        clrc
1e8d: c7 30     mov   ($30+x),a
1e8f: 4e 94 18  tclr1 $1894
1e92: c6        mov   (x),a
1e93: 0c 94 94  asl   $9494
1e96: 60        clrc
1e97: c7 30     mov   ($30+x),a
1e99: 8d 18     mov   y,#$18
1e9b: c6        mov   (x),a
1e9c: 0c 8d 8d  asl   $8d8d
1e9f: 0c 6c c7  asl   $c76c
1ea2: d8 0c     mov   $0c,x
1ea4: 6f        ret
1ea5: d0 d0     bne   $1e77
1ea7: 0c 7c d8  asl   $d87c
1eaa: 0c 7f d0  asl   $d07f
1ead: 0c d8 d8  asl   $d8d8
1eb0: da 05     movw  $05,ya
1eb2: db 0a     mov   $0a+x,y
1eb4: de 00 00  cbne  $00+x,$1eb7
1eb7: 00        nop
1eb8: e9 bc 23  mov   x,$23bc
1ebb: 08 0c     or    a,#$0c
1ebd: 49 d1 d1  eor   ($d1),($d1)
1ec0: 0c 4b d1  asl   $d14b
1ec3: 0c 49 d1  asl   $d149
1ec6: 0c 4c d1  asl   $d14c
1ec9: 0c 49 d1  asl   $d149
1ecc: 0c 4b d1  asl   $d14b
1ecf: 0c 49 d1  asl   $d149
1ed2: 00        nop
1ed3: da 05     movw  $05,ya
1ed5: db 0a     mov   $0a+x,y
1ed7: de 00 00  cbne  $00+x,$1eda
1eda: 00        nop
1edb: 30 4f     bmi   $1f2c
1edd: d0 02     bne   $1ee1
1edf: c7 16     mov   ($16+x),a
1ee1: d8 0c     mov   $0c,x
1ee3: c6        mov   (x),a
1ee4: d0 0c     bne   $1ef2
1ee6: c7 d0     mov   ($d0+x),a
1ee8: c7 d0     mov   ($d0+x),a
1eea: 01        tcall 0
1eeb: c7 17     mov   ($17+x),a
1eed: d8 0c     mov   $0c,x
1eef: c6        mov   (x),a
1ef0: d0 30     bne   $1f22
1ef2: d0 02     bne   $1ef6
1ef4: c7 16     mov   ($16+x),a
1ef6: d8 0c     mov   $0c,x
1ef8: c6        mov   (x),a
1ef9: d0 0c     bne   $1f07
1efb: 4d        push  x
1efc: c7 d0     mov   ($d0+x),a
1efe: 0c 4e d0  asl   $d04e
1f01: d0 01     bne   $1f04
1f03: c7 17     mov   ($17+x),a
1f05: d8 0c     mov   $0c,x
1f07: c6        mov   (x),a
1f08: d0 da     bne   $1ee4
1f0a: 05 db 0a  or    a,$0adb
1f0d: de 00 00  cbne  $00+x,$1f10
1f10: 00        nop
1f11: 30 4f     bmi   $1f62
1f13: d0 02     bne   $1f17
1f15: c7 16     mov   ($16+x),a
1f17: d8 0c     mov   $0c,x
1f19: c6        mov   (x),a
1f1a: d0 0c     bne   $1f28
1f1c: c7 d0     mov   ($d0+x),a
1f1e: c7 d0     mov   ($d0+x),a
1f20: 01        tcall 0
1f21: c7 17     mov   ($17+x),a
1f23: d8 0c     mov   $0c,x
1f25: c6        mov   (x),a
1f26: d0 30     bne   $1f58
1f28: d0 03     bne   $1f2d
1f2a: c7 15     mov   ($15+x),a
1f2c: d8 0c     mov   $0c,x
1f2e: c6        mov   (x),a
1f2f: d0 0c     bne   $1f3d
1f31: 7f        reti
1f32: c7 03     mov   ($03+x),a
1f34: c6        mov   (x),a
1f35: 09 d8 0c  or    ($0c),($d8)
1f38: d0 d0     bne   $1f0a
1f3a: 03 c7 09  bbs0  $c7,$1f46
1f3d: d8 0c     mov   $0c,x
1f3f: d0 03     bne   $1f44
1f41: c6        mov   (x),a
1f42: 09 d8 03  or    ($03),($d8)
1f45: c6        mov   (x),a
1f46: 09 d8 f0  or    ($f0),($d8)
1f49: da 03     movw  $03,ya
1f4b: e2 3c     set7  $3c
1f4d: e0        clrv
1f4e: e6        mov   a,(x)
1f4f: db 0f     mov   $0f+x,y
1f51: 18 3e b2  or    $b2,#$3e
1f54: b2 b2     clr5  $b2
1f56: a6        sbc   a,(x)
1f57: b2 b2     clr5  $b2
1f59: b2 a6     clr5  $a6
1f5b: 06        or    a,(x)
1f5c: b3 b0 b3  bbc5  $b0,$1f12
1f5f: b0 b3     bcs   $1f14
1f61: b0 b3     bcs   $1f16
1f63: b0 b3     bcs   $1f18
1f65: b0 b3     bcs   $1f1a
1f67: b0 b3     bcs   $1f1c
1f69: b0 b3     bcs   $1f1e
1f6b: b0 18     bcs   $1f85
1f6d: b2 ad     clr5  $ad
1f6f: a6        sbc   a,(x)
1f70: 00        nop
1f71: da 00     movw  $00,ya
1f73: db 0a     mov   $0a+x,y
1f75: e7 fa     mov   a,($fa+x)
1f77: de 23 13  cbne  $23+x,$1f8d
1f7a: 40        setp
1f7b: 18 4e be  or    $be,#$4e
1f7e: be        das   a
1f7f: be        das   a
1f80: b2 be     clr5  $be
1f82: be        das   a
1f83: be        das   a
1f84: b2 e1     clr5  $e1
1f86: 0c 96 0c  asl   $0c96
1f89: bf        mov   a,(x)+
1f8a: e1        tcall 14
1f8b: 54 fa     eor   a,$fa+x
1f8d: 54 c6     eor   a,$c6+x
1f8f: e0        clrv
1f90: e6        mov   a,(x)
1f91: 18 be c7  or    $c7,#$be
1f94: c7 da     mov   ($da+x),a
1f96: 01        tcall 0
1f97: db 0a     mov   $0a+x,y
1f99: e7 fa     mov   a,($fa+x)
1f9b: de 23 12  cbne  $23+x,$1fb0
1f9e: 40        setp
1f9f: 18 4e a6  or    $a6,#$4e
1fa2: a6        sbc   a,(x)
1fa3: a6        sbc   a,(x)
1fa4: 9a a6     subw  ya,$a6
1fa6: a6        sbc   a,(x)
1fa7: a6        sbc   a,(x)
1fa8: 9a 60     subw  ya,$60
1faa: ad 18     cmp   y,#$18
1fac: aa c7 c7  mov1  c,$07c7,6
1faf: da 00     movw  $00,ya
1fb1: db 0a     mov   $0a+x,y
1fb3: e7 fa     mov   a,($fa+x)
1fb5: de 23 11  cbne  $23+x,$1fc9
1fb8: 40        setp
1fb9: 60        clrc
1fba: 4e c7 c7  tclr1 $c7c7
1fbd: 60        clrc
1fbe: 9d        mov   x,sp
1fbf: 18 9a c7  or    $c7,#$9a
1fc2: c7 da     mov   ($da+x),a
1fc4: 02 e0     set0  $e0
1fc6: c8 db     cmp   x,#$db
1fc8: 00        nop
1fc9: 18 4b b2  or    $b2,#$4b
1fcc: bb bb     inc   $bb+x
1fce: bb b2     inc   $b2+x
1fd0: bb bb     inc   $bb+x
1fd2: bb b2     inc   $b2+x
1fd4: bb bb     inc   $bb+x
1fd6: bb 0c     inc   $0c+x
1fd8: bb bc     inc   $bc+x
1fda: 18 bb c6  or    $c6,#$bb
1fdd: b9        sbc   (x),(y)
1fde: b2 b9     clr5  $b9
1fe0: b9        sbc   (x),(y)
1fe1: 0c b9 c6  asl   $c6b9
1fe4: 18 b2 b9  or    $b9,#$b2
1fe7: b9        sbc   (x),(y)
1fe8: 0c b9 c6  asl   $c6b9
1feb: 18 b2 b9  or    $b9,#$b2
1fee: b9        sbc   (x),(y)
1fef: 0c b9 c6  asl   $c6b9
1ff2: b9        sbc   (x),(y)
1ff3: bb 18     inc   $18+x
1ff5: b9        sbc   (x),(y)
1ff6: c6        mov   (x),a
1ff7: b7 00     sbc   a,($00)+y
1ff9: da 00     movw  $00,ya
1ffb: db 0a     mov   $0a+x,y
1ffd: e7 dc     mov   a,($dc+x)
1fff: de 23 13  cbne  $23+x,$2015
2002: 40        setp
2003: 18 4c b2  or    $b2,#$4c
2006: bb bb     inc   $bb+x
2008: 0c 4d bb  asl   $bb4d
200b: c7 18     mov   ($18+x),a
200d: 4c b2 bb  lsr   $bbb2
2010: bb 0c     inc   $0c+x
2012: 4d        push  x
2013: bb c7     inc   $c7+x
2015: 18 4c b2  or    $b2,#$4c
2018: bb bb     inc   $bb+x
201a: 0c 4d bb  asl   $bb4d
201d: c7 0c     mov   ($0c+x),a
201f: 4c bb bc  lsr   $bcbb
2022: 18 bb c6  or    $c6,#$bb
2025: b9        sbc   (x),(y)
2026: b2 b9     clr5  $b9
2028: b9        sbc   (x),(y)
2029: 0c 4d b9  asl   $b94d
202c: c7 18     mov   ($18+x),a
202e: 4c b2 b9  lsr   $b9b2
2031: b9        sbc   (x),(y)
2032: 0c 4d b9  asl   $b94d
2035: c7 18     mov   ($18+x),a
2037: 4c b2 b9  lsr   $b9b2
203a: b9        sbc   (x),(y)
203b: 0c 4d b9  asl   $b94d
203e: c7 0c     mov   ($0c+x),a
2040: 4c b9 bb  lsr   $bbb9
2043: 18 b9 c6  or    $c6,#$b9
2046: b7 da     sbc   a,($da)+y
2048: 01        tcall 0
2049: db 0a     mov   $0a+x,y
204b: 18 6e c7  or    $c7,#$6e
204e: 93 ab ab  bbc4  $ab,$1ffc
2051: c7 93     mov   ($93+x),a
2053: ab ab     inc   $ab
2055: c7 93     mov   ($93+x),a
2057: ab ab     inc   $ab
2059: c7 8e     mov   ($8e+x),a
205b: aa aa c7  mov1  c,$07aa,6
205e: 8e        pop   psw
205f: aa aa c7  mov1  c,$07aa,6
2062: 8e        pop   psw
2063: aa aa c7  mov1  c,$07aa,6
2066: 8e        pop   psw
2067: aa c7 aa  mov1  c,$0ac7,5
206a: 93 ab ab  bbc4  $ab,$2018
206d: da 01     movw  $01,ya
206f: db 0f     mov   $0f+x,y
2071: 18 6e c7  or    $c7,#$6e
2074: c7 a3     mov   ($a3+x),a
2076: a3 c7 c7  bbs5  $c7,$2040
2079: a3 a3 c7  bbs5  $a3,$2043
207c: c7 a3     mov   ($a3+x),a
207e: a3 c7 c7  bbs5  $c7,$2048
2081: a1        tcall 10
2082: a1        tcall 10
2083: c7 c7     mov   ($c7+x),a
2085: a1        tcall 10
2086: a1        tcall 10
2087: c7 c7     mov   ($c7+x),a
2089: a1        tcall 10
208a: a1        tcall 10
208b: c7 c7     mov   ($c7+x),a
208d: a1        tcall 10
208e: c7 a1     mov   ($a1+x),a
2090: c7 a3     mov   ($a3+x),a
2092: a3 da 03  bbs5  $da,$2098
2095: db 0f     mov   $0f+x,y
2097: 18 6e c7  or    $c7,#$6e
209a: c7 9a     mov   ($9a+x),a
209c: 9f        xcn   a
209d: c7 c7     mov   ($c7+x),a
209f: 9a 9f     subw  ya,$9f
20a1: c7 c7     mov   ($c7+x),a
20a3: 9a a3     subw  ya,$a3
20a5: c7 a1     mov   ($a1+x),a
20a7: c7 a1     mov   ($a1+x),a
20a9: c7 c7     mov   ($c7+x),a
20ab: 9a a1     subw  ya,$a1
20ad: c7 c7     mov   ($c7+x),a
20af: 9a a1     subw  ya,$a1
20b1: c7 c7     mov   ($c7+x),a
20b3: 9a 9e     subw  ya,$9e
20b5: c7 9f     mov   ($9f+x),a
20b7: c7 9f     mov   ($9f+x),a
20b9: da 01     movw  $01,ya
20bb: db 05     mov   $05+x,y
20bd: de 23 12  cbne  $23+x,$20d2
20c0: 40        setp
20c1: 18 6c c7  or    $c7,#$6c
20c4: e0        clrv
20c5: c8 e1     cmp   x,#$e1
20c7: ff        stop
20c8: f0 60     beq   $212a
20ca: b7 b5     sbc   a,($b5)+y
20cc: b4 b3     sbc   a,$b3+x
20ce: e1        tcall 14
20cf: 64 c8     cmp   a,$c8
20d1: b2 b0     clr5  $b0
20d3: af        mov   (x)+,a
20d4: 48 c6     eor   a,#$c6
20d6: c7 da     mov   ($da+x),a
20d8: 02 db     set0  $db
20da: 00        nop
20db: 18 4b b2  or    $b2,#$4b
20de: bb bb     inc   $bb+x
20e0: bb b2     inc   $b2+x
20e2: bb bb     inc   $bb+x
20e4: bb b2     inc   $b2+x
20e6: bb bb     inc   $bb+x
20e8: bb 0c     inc   $0c+x
20ea: b9        sbc   (x),(y)
20eb: bb 18     inc   $18+x
20ed: bc        inc   a
20ee: c6        mov   (x),a
20ef: c0        di
20f0: c6        mov   (x),a
20f1: be        das   a
20f2: be        das   a
20f3: be        das   a
20f4: b2 bc     clr5  $bc
20f6: bc        inc   a
20f7: bc        inc   a
20f8: b6 b7 c6  sbc   a,$c6b7+y
20fb: c6        mov   (x),a
20fc: c7 c7     mov   ($c7+x),a
20fe: c7 00     mov   ($00+x),a
2100: da 00     movw  $00,ya
2102: db 0a     mov   $0a+x,y
2104: 18 4c b2  or    $b2,#$4c
2107: bb bb     inc   $bb+x
2109: 0c 4d bb  asl   $bb4d
210c: c7 18     mov   ($18+x),a
210e: 4c b2 bb  lsr   $bbb2
2111: bb 0c     inc   $0c+x
2113: 4d        push  x
2114: bb c7     inc   $c7+x
2116: 18 4c b2  or    $b2,#$4c
2119: bb bb     inc   $bb+x
211b: 0c 4d bb  asl   $bb4d
211e: c7 0c     mov   ($0c+x),a
2120: 4c b9 bb  lsr   $bbb9
2123: 30 bc     bmi   $20e1
2125: c0        di
2126: 18 be be  or    $be,#$be
2129: 0c 4d be  asl   $be4d
212c: c7 18     mov   ($18+x),a
212e: 4c b2 bc  lsr   $bcb2
2131: bc        inc   a
2132: 0c 4d bc  asl   $bc4d
2135: c7 18     mov   ($18+x),a
2137: 4c b6 b7  lsr   $b7b6
213a: c6        mov   (x),a
213b: c6        mov   (x),a
213c: c7 c7     mov   ($c7+x),a
213e: c7 da     mov   ($da+x),a
2140: 01        tcall 0
2141: db 0a     mov   $0a+x,y
2143: 18 6e c7  or    $c7,#$6e
2146: 93 ab ab  bbc4  $ab,$20f4
2149: c7 97     mov   ($97+x),a
214b: af        mov   (x)+,a
214c: af        mov   (x)+,a
214d: c7 98     mov   ($98+x),a
214f: b0 b0     bcs   $2101
2151: c7 99     mov   ($99+x),a
2153: b1        tcall 11
2154: b1        tcall 11
2155: c7 9a     mov   ($9a+x),a
2157: ad ad     cmp   y,#$ad
2159: c7 92     mov   ($92+x),a
215b: aa aa c7  mov1  c,$07aa,6
215e: 93 c7 8e  bbc4  $c7,$20ef
2161: c7 93     mov   ($93+x),a
2163: c7 00     mov   ($00+x),a
2165: da 01     movw  $01,ya
2167: db 0f     mov   $0f+x,y
2169: 18 6e c7  or    $c7,#$6e
216c: c7 a3     mov   ($a3+x),a
216e: a3 c7 c7  bbs5  $c7,$2138
2171: a9 a9 c7  sbc   ($c7),($a9)
2174: c7 ab     mov   ($ab+x),a
2176: ab c7     inc   $c7
2178: c7 ab     mov   ($ab+x),a
217a: ab c7     inc   $c7
217c: c7 a6     mov   ($a6+x),a
217e: a6        sbc   a,(x)
217f: c7 c7     mov   ($c7+x),a
2181: a4 a4     sbc   a,$a4
2183: c7 a3     mov   ($a3+x),a
2185: c7 9f     mov   ($9f+x),a
2187: c7 a3     mov   ($a3+x),a
2189: c7 00     mov   ($00+x),a
218b: da 03     movw  $03,ya
218d: db 0f     mov   $0f+x,y
218f: 18 6e c7  or    $c7,#$6e
2192: c7 9a     mov   ($9a+x),a
2194: 9f        xcn   a
2195: c7 c7     mov   ($c7+x),a
2197: 9d        mov   x,sp
2198: a3 c7 c7  bbs5  $c7,$2162
219b: 9f        xcn   a
219c: a4 c7     sbc   a,$c7
219e: a5 c7 a5  sbc   a,$a5c7
21a1: c7 c7     mov   ($c7+x),a
21a3: a1        tcall 10
21a4: a6        sbc   a,(x)
21a5: c7 c7     mov   ($c7+x),a
21a7: 9e        div   ya,x
21a8: a4 c7     sbc   a,$c7
21aa: 9a c7     subw  ya,$c7
21ac: 97 c7     adc   a,($c7)+y
21ae: 93 c7 da  bbc4  $c7,$218b
21b1: 00        nop
21b2: db 05     mov   $05+x,y
21b4: de 23 12  cbne  $23+x,$21c9
21b7: 40        setp
21b8: 24 6d     and   a,$6d
21ba: af        mov   (x)+,a
21bb: 0c ad 60  asl   $60ad
21be: af        mov   (x)+,a
21bf: b2 b0     clr5  $b0
21c1: b4 30     sbc   a,$30+x
21c3: b6 b7 b9  sbc   a,$b9b7+y
21c6: bc        inc   a
21c7: 60        clrc
21c8: bb 30     inc   $30+x
21ca: c7 00     mov   ($00+x),a
21cc: da 01     movw  $01,ya
21ce: e7 f0     mov   a,($f0+x)
21d0: db 0a     mov   $0a+x,y
21d2: de 23 13  cbne  $23+x,$21e8
21d5: 40        setp
21d6: 24 6e     and   a,$6e
21d8: b2 0c     clr5  $0c
21da: b0 30     bcs   $220c
21dc: af        mov   (x)+,a
21dd: b2 b7     clr5  $b7
21df: 18 c6 b6  or    $b6,#$c6
21e2: 30 b3     bmi   $2197
21e4: b4 b9     sbc   a,$b9+x
21e6: 18 c6 b7  or    $b7,#$c6
21e9: da 03     movw  $03,ya
21eb: db 0f     mov   $0f+x,y
21ed: 06        or    a,(x)
21ee: b6 ad b6  sbc   a,$b6ad+y
21f1: ad b6     cmp   y,#$b6
21f3: ad b6     cmp   y,#$b6
21f5: c7 b7     mov   ($b7+x),a
21f7: af        mov   (x)+,a
21f8: b7 af     sbc   a,($af)+y
21fa: b7 af     sbc   a,($af)+y
21fc: b7 c7     sbc   a,($c7)+y
21fe: b9        sbc   (x),(y)
21ff: b0 b9     bcs   $21ba
2201: b0 b9     bcs   $21bc
2203: b0 b9     bcs   $21be
2205: c7 bc     mov   ($bc+x),a
2207: b4 bc     sbc   a,$bc+x
2209: b4 bc     sbc   a,$bc+x
220b: b4 bc     sbc   a,$bc+x
220d: c7 bb     mov   ($bb+x),a
220f: b2 bb     clr5  $bb
2211: b2 bb     clr5  $bb
2213: b2 bb     clr5  $bb
2215: b2 bb     clr5  $bb
2217: b2 bb     clr5  $bb
2219: b2 bb     clr5  $bb
221b: b2 bb     clr5  $bb
221d: b2 e8     clr5  $e8
221f: 30 60     bmi   $2281
2221: 06        or    a,(x)
2222: bb b2     inc   $b2+x
2224: bb b2     inc   $b2+x
2226: bb b2     inc   $b2+x
2228: bb b2     inc   $b2+x
222a: da 01     movw  $01,ya
222c: db 14     mov   $14+x,y
222e: 30 c7     bmi   $21f7
2230: 18 9f a3  or    $a3,#$9f
2233: a6        sbc   a,(x)
2234: a3 9f a3  bbs5  $9f,$21da
2237: a6        sbc   a,(x)
2238: a3 a4 a8  bbs5  $a4,$21e3
223b: ab a8     inc   $a8
223d: a5 a8 ab  sbc   a,$aba8
2240: a8 a6     sbc   a,#$a6
2242: aa ad aa  mov1  c,$0aad,5
2245: a6        sbc   a,(x)
2246: aa ad aa  mov1  c,$0aad,5
2249: 9f        xcn   a
224a: a3 a6 a3  bbs5  $a6,$21f0
224d: 9f        xcn   a
224e: a3 a6 a3  bbs5  $a6,$21f4
2251: da 04     movw  $04,ya
2253: de 23 11  cbne  $23+x,$2267
2256: 40        setp
2257: db 0a     mov   $0a+x,y
2259: 30 6c     bmi   $22c7
225b: c7 60     mov   ($60+x),a
225d: 93 97 98  bbc4  $97,$21f8
2260: 99        adc   (x),(y)
2261: 9a 95     subw  ya,$95
2263: 97 93     adc   a,($93)+y
2265: da 00     movw  $00,ya
2267: db 05     mov   $05+x,y
2269: de 23 12  cbne  $23+x,$227e
226c: 40        setp
226d: 24 6d     and   a,$6d
226f: af        mov   (x)+,a
2270: 0c ad 60  asl   $60ad
2273: af        mov   (x)+,a
2274: b2 b0     clr5  $b0
2276: b4 30     sbc   a,$30+x
2278: b6 18 b5  sbc   a,$b518+y
227b: b6 48 bc  sbc   a,$bc48+y
227e: 18 b6 60  or    $60,#$b6
2281: b7 48     sbc   a,($48)+y
2283: c7 00     mov   ($00+x),a
2285: da 01     movw  $01,ya
2287: db 0a     mov   $0a+x,y
2289: e7 fa     mov   a,($fa+x)
228b: 24 6e     and   a,$6e
228d: b2 0c     clr5  $0c
228f: b0 30     bcs   $22c1
2291: af        mov   (x)+,a
2292: b2 b7     clr5  $b7
2294: 18 c6 b6  or    $b6,#$c6
2297: 30 b3     bmi   $224c
2299: b4 b9     sbc   a,$b9+x
229b: 18 c6 b7  or    $b7,#$c6
229e: da 03     movw  $03,ya
22a0: db 0f     mov   $0f+x,y
22a2: 06        or    a,(x)
22a3: b6 ad b6  sbc   a,$b6ad+y
22a6: ad b6     cmp   y,#$b6
22a8: ad b6     cmp   y,#$b6
22aa: c7 b5     mov   ($b5+x),a
22ac: ad b5     cmp   y,#$b5
22ae: c7 b6     mov   ($b6+x),a
22b0: ad b6     cmp   y,#$b6
22b2: c7 bc     mov   ($bc+x),a
22b4: b2 bc     clr5  $bc
22b6: b2 bc     clr5  $bc
22b8: b2 bc     clr5  $bc
22ba: b2 bc     clr5  $bc
22bc: b2 bc     clr5  $bc
22be: c7 b6     mov   ($b6+x),a
22c0: ad b6     cmp   y,#$b6
22c2: c7 b7     mov   ($b7+x),a
22c4: af        mov   (x)+,a
22c5: b7 af     sbc   a,($af)+y
22c7: b7 af     sbc   a,($af)+y
22c9: b7 af     sbc   a,($af)+y
22cb: b7 af     sbc   a,($af)+y
22cd: b7 af     sbc   a,($af)+y
22cf: b7 af     sbc   a,($af)+y
22d1: b7 af     sbc   a,($af)+y
22d3: e8 30     mov   a,#$30
22d5: 60        clrc
22d6: 06        or    a,(x)
22d7: b7 af     sbc   a,($af)+y
22d9: b7 af     sbc   a,($af)+y
22db: b7 af     sbc   a,($af)+y
22dd: b7 af     sbc   a,($af)+y
22df: c7 c7     mov   ($c7+x),a
22e1: c7 c7     mov   ($c7+x),a
22e3: da 01     movw  $01,ya
22e5: db 14     mov   $14+x,y
22e7: 30 c7     bmi   $22b0
22e9: 18 9f a3  or    $a3,#$9f
22ec: a6        sbc   a,(x)
22ed: a3 a3 a6  bbs5  $a3,$2296
22f0: a9 a6 a4  sbc   ($a4),($a6)
22f3: a8 ab     sbc   a,#$ab
22f5: a8 a5     sbc   a,#$a5
22f7: a8 ab     sbc   a,#$ab
22f9: a8 a6     sbc   a,#$a6
22fb: aa ad aa  mov1  c,$0aad,5
22fe: a1        tcall 10
22ff: a6        sbc   a,(x)
2300: aa a6 a3  mov1  c,$03a6,5
2303: a6        sbc   a,(x)
2304: a1        tcall 10
2305: a6        sbc   a,(x)
2306: 9f        xcn   a
2307: c6        mov   (x),a
2308: c6        mov   (x),a
2309: da 04     movw  $04,ya
230b: db 0a     mov   $0a+x,y
230d: de 23 11  cbne  $23+x,$2321
2310: 40        setp
2311: 30 6d     bmi   $2380
2313: c7 60     mov   ($60+x),a
2315: 93 97 98  bbc4  $97,$22b0
2318: 99        adc   (x),(y)
2319: 9a 92     subw  ya,$92
231b: 93 c6 da  bbc4  $c6,$22f8
231e: 00        nop
231f: e0        clrv
2320: dc        dec   y
2321: db 0a     mov   $0a+x,y
2323: de 23 12  cbne  $23+x,$2338
2326: 40        setp
2327: 18 6e b3  or    $b3,#$6e
232a: b4 b4     sbc   a,$b4+x
232c: b4 b6     sbc   a,$b6+x
232e: b7 c6     sbc   a,($c6)+y
2330: bc        inc   a
2331: c6        mov   (x),a
2332: da 01     movw  $01,ya
2334: e0        clrv
2335: aa 10 af  mov1  c,$0f10,5
2338: b0 af     bcs   $22e9
233a: ad af     cmp   y,#$af
233c: ad 30     cmp   y,#$30
233e: ab 18     inc   $18
2340: a6        sbc   a,(x)
2341: 00        nop
2342: da 02     movw  $02,ya
2344: db 05     mov   $05+x,y
2346: e7 fa     mov   a,($fa+x)
2348: 18 6e c7  or    $c7,#$6e
234b: 06        or    a,(x)
234c: 98 9a 9c  adc   $9c,#$9a
234f: 9d        mov   x,sp
2350: 9f        xcn   a
2351: a1        tcall 10
2352: a3 a4 a6  bbs5  $a4,$22fb
2355: a8 a9     sbc   a,#$a9
2357: ab ad     inc   $ad
2359: af        mov   (x)+,a
235a: b0 c6     bcs   $2322
235c: 60        clrc
235d: c7 da     mov   ($da+x),a
235f: 01        tcall 0
2360: db 0a     mov   $0a+x,y
2362: 10 ab     bpl   $230f
2364: ad ab     cmp   y,#$ab
2366: aa ab aa  mov1  c,$0aab,5
2369: 30 a6     bmi   $2311
236b: 18 a3 da  or    $da,#$a3
236e: 01        tcall 0
236f: db 0a     mov   $0a+x,y
2371: 18 6e c7  or    $c7,#$6e
2374: 48 9f     eor   a,#$9f
2376: 18 9f 30  or    $30,#$9f
2379: 9f        xcn   a
237a: c7 60     mov   ($60+x),a
237c: 9f        xcn   a
237d: 9a da     subw  ya,$da
237f: 01        tcall 0
2380: db 0a     mov   $0a+x,y
2382: 18 6e c7  or    $c7,#$6e
2385: 48 98     eor   a,#$98
2387: 18 98 30  or    $30,#$98
238a: 98 c7 60  adc   $60,#$c7
238d: 97 93     adc   a,($93)+y
238f: da 00     movw  $00,ya
2391: e0        clrv
2392: dc        dec   y
2393: db 0a     mov   $0a+x,y
2395: de 23 12  cbne  $23+x,$23aa
2398: 40        setp
2399: 18 6e b3  or    $b3,#$6e
239c: b4 b4     sbc   a,$b4+x
239e: b4 b6     sbc   a,$b6+x
23a0: b7 c6     sbc   a,($c6)+y
23a2: bc        inc   a
23a3: c6        mov   (x),a
23a4: da 01     movw  $01,ya
23a6: 18 2e b2  or    $b2,#$2e
23a9: b1        tcall 11
23aa: b2 ad     clr5  $ad
23ac: a6        sbc   a,(x)
23ad: a5 a6 a1  sbc   a,$a1a6
23b0: e0        clrv
23b1: 96 e1 60  adc   a,$60e1+y
23b4: fa 60 5e  mov   ($5e),($60)
23b7: a7 da     sbc   a,($da+x)
23b9: 03 db 0f  bbs0  $db,$23cb
23bc: 18 1e a6  or    $a6,#$1e
23bf: ad b2     cmp   y,#$b2
23c1: b9        sbc   (x),(y)
23c2: be        das   a
23c3: c7 c7     mov   ($c7+x),a
23c5: c7 da     mov   ($da+x),a
23c7: 01        tcall 0
23c8: db 0a     mov   $0a+x,y
23ca: 48 5d     eor   a,#$5d
23cc: a6        sbc   a,(x)
23cd: 00        nop
23ce: da 02     movw  $02,ya
23d0: db 05     mov   $05+x,y
23d2: 18 6e c7  or    $c7,#$6e
23d5: 06        or    a,(x)
23d6: 98 9a 9c  adc   $9c,#$9a
23d9: 9d        mov   x,sp
23da: 9f        xcn   a
23db: a1        tcall 10
23dc: a3 a4 a6  bbs5  $a4,$2385
23df: a8 a9     sbc   a,#$a9
23e1: ab ad     inc   $ad
23e3: af        mov   (x)+,a
23e4: b0 c6     bcs   $23ac
23e6: 60        clrc
23e7: c7 da     mov   ($da+x),a
23e9: 01        tcall 0
23ea: db 0a     mov   $0a+x,y
23ec: 18 ad c7  or    $c7,#$ad
23ef: c7 ad     mov   ($ad+x),a
23f1: ad c7     cmp   y,#$c7
23f3: c7 ad     mov   ($ad+x),a
23f5: 9b c6     dec   $c6+x
23f7: c6        mov   (x),a
23f8: c6        mov   (x),a
23f9: da 00     movw  $00,ya
23fb: db 05     mov   $05+x,y
23fd: 18 1e a6  or    $a6,#$1e
2400: ad b2     cmp   y,#$b2
2402: b9        sbc   (x),(y)
2403: be        das   a
2404: c7 c7     mov   ($c7+x),a
2406: c7 da     mov   ($da+x),a
2408: 01        tcall 0
2409: 48 5d     eor   a,#$5d
240b: a1        tcall 10
240c: da 01     movw  $01,ya
240e: db 0a     mov   $0a+x,y
2410: 18 6e c7  or    $c7,#$6e
2413: 48 9f     eor   a,#$9f
2415: 18 9f 30  or    $30,#$9f
2418: 9f        xcn   a
2419: c7 18     mov   ($18+x),a
241b: a1        tcall 10
241c: c7 c7     mov   ($c7+x),a
241e: c7 a1     mov   ($a1+x),a
2420: c7 c7     mov   ($c7+x),a
2422: c7 60     mov   ($60+x),a
2424: 9d        mov   x,sp
2425: 18 9e c7  or    $c7,#$9e
2428: c7 c7     mov   ($c7+x),a
242a: c7 c7     mov   ($c7+x),a
242c: c7 c7     mov   ($c7+x),a
242e: 48 5d     eor   a,#$5d
2430: 9e        div   ya,x
2431: da 01     movw  $01,ya
2433: db 14     mov   $14+x,y
2435: 18 6e c7  or    $c7,#$6e
2438: 48 c7     eor   a,#$c7
243a: 18 c7 30  or    $30,#$c7
243d: c7 c7     mov   ($c7+x),a
243f: 18 9e c7  or    $c7,#$9e
2442: c7 c7     mov   ($c7+x),a
2444: 9e        div   ya,x
2445: c7 c7     mov   ($c7+x),a
2447: c7 60     mov   ($60+x),a
2449: a4 18     sbc   a,$18
244b: a1        tcall 10
244c: c7 c7     mov   ($c7+x),a
244e: c7 c7     mov   ($c7+x),a
2450: c7 c7     mov   ($c7+x),a
2452: c7 48     mov   ($48+x),a
2454: 5d        mov   x,a
2455: a4 00     sbc   a,$00
2457: da 01     movw  $01,ya
2459: db 0a     mov   $0a+x,y
245b: 18 6e c7  or    $c7,#$6e
245e: 48 98     eor   a,#$98
2460: 18 98 30  or    $30,#$98
2463: 98 c7 18  adc   $18,#$c7
2466: 9a c7     subw  ya,$c7
2468: c7 c7     mov   ($c7+x),a
246a: 9a c7     subw  ya,$c7
246c: c7 c7     mov   ($c7+x),a
246e: 60        clrc
246f: a1        tcall 10
2470: 18 9a c7  or    $c7,#$9a
2473: c7 c7     mov   ($c7+x),a
2475: c7 c7     mov   ($c7+x),a
2477: c7 c7     mov   ($c7+x),a
2479: 48 6e     eor   a,#$6e
247b: 8e        pop   psw
247c: 00        nop
247d: 00        nop
247e: 00        nop
247f: 00        nop
