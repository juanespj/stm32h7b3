MEMORY
{
    FLASH   : ORIGIN = 0x08000000, LENGTH = 64K   /* Bootloader region */
    RAM     : ORIGIN = 0x24000000, LENGTH = 1024K  /* AXI SRAM is 1MB on H7B3 */
}
