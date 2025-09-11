pub fn generate_index_for_quad(counter: usize, index_buffer_data: &mut Vec<u16>) {
    let num = counter as u16;
    index_buffer_data.push(num * 4);
    index_buffer_data.push(num * 4 + 1);
    index_buffer_data.push(num * 4 + 2);
    index_buffer_data.push(num * 4 + 1);
    index_buffer_data.push(num * 4 + 3);
    index_buffer_data.push(num * 4 + 2);
}
