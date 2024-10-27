//Chat GPT generated
pub fn set_bits_in_range(original: u32, start: u8, end: u8, new_value: u32) -> u32 {
    // Calculate the length of the bit range
    let bit_length = end - start + 1;

    // Create a mask for the specified range
    let range_mask = ((1 << bit_length) - 1) << start;

    // Clear the range in the original number
    let cleared_original = original & !range_mask;

    // Shift the new value to the correct position and mask it
    let shifted_value = (new_value << start) & range_mask;

    // Combine the cleared original number with the shifted new value
    cleared_original | shifted_value
}