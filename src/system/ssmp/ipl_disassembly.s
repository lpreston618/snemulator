$FFC0:  CD EF    ldx   #$EF
$FFC2:  BD       txs   
$FFC3:  E8 00    lda   #$00
$FFC5:  C6       sta   (X)
$FFC6:  1D       dex   
$FFC7:  D0 FC    bne   $FFC5
$FFC9:  8F AA F4 mov   $F4,#$AA
$FFCC:  8F BB F5 mov   $F5,#$BB
$FFCF:  78 CC F4 cmp   $F4,#$CC
$FFD2:  D0 FB    bne   $FFCF
$FFD4:  2F 19    bra   $FFEF
$FFD6:  EB F4    ldy   $F4
$FFD8:  D0 FC    bne   $FFD6
$FFDA:  7E F4    cmy   $F4
$FFDC:  D0 0B    bne   $FFE9
$FFDE:  E4 F5    lda   $F5
$FFE0:  CB F4    sty   $F4
$FFE2:  D7 00    sta   ($00)+Y
$FFE4:  FC       iny   
$FFE5:  D0 F3    bne   $FFDA
$FFE7:  AB 01    inc   $01
$FFE9:  10 EF    bpl   $FFDA
$FFEB:  7E F4    cmy   $F4
$FFED:  10 EB    bpl   $FFDA
$FFEF:  BA F6    ldya  $F6
$FFF1:  DA 00    stya  $00
$FFF3:  BA F4    ldya  $F4
$FFF5:  C4 F4    sta   $F4
$FFF7:  DD       tya   
$FFF8:  5D       tax   
$FFF9:  D0 DB    bne   $FFD6
$FFFB:  1F 00 00 jmp   ($0000+X)
$FFFE:  C0       cli   
$FFFF:  FF       stop  
