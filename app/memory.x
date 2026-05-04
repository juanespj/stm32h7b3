MEMORY
{
    FLASH   : ORIGIN = 0x08010000, LENGTH = 1984K  /* After 64KB bootloader */
    RAM     : ORIGIN = 0x24000000, LENGTH = 1024K  /* AXI SRAM is 1MB on H7B3 */
}
