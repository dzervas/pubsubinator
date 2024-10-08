MEMORY
{
  FLASH : ORIGIN = 0x00000000 + 156K, LENGTH = 1024K - 156K

  /*
  BOOT    (rx) : ORIGIN = 0x00000000, LENGTH = 0x0014000
  FLASH   (rx) : ORIGIN = 0x00014000, LENGTH = 0x00063000
  SLOT1   (rx) : ORIGIN = 0x00077000, LENGTH = 0x00063000
  STORAGE (rx) : ORIGIN = 0x000F8000, LENGTH = 0x00008000
  */

  RAM     (rw) : ORIGIN = 0x20000000 + 0xFA18, LENGTH = 256K - 0xFA18
}
