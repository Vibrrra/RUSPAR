use ndarray::Array2;
use num_traits::Float;

pub fn create_hadamard(inputs: usize) -> Array2<f32> {
    //let n = (2i32).pow(n as u32);

    let shape = ndarray::Ix2(inputs, inputs);
    let mut hadamard = Array2::<f32>::zeros(shape);

    // algorithm starts here

    hadamard[[0, 0]] = 1.0;

    let mut k = 1;
    while k < inputs {
        let mut i = 0;
        while i < k {
            let mut j = 0;
            while j < k {
                hadamard[[i + k, j]] = -hadamard[[i, j]];
                hadamard[[i, j + k]] = hadamard[[i, j]];
                hadamard[[i + k, j + k]] = hadamard[[i, j]];
                j += 1;
            }
            i += 1;
        }
        k += k;
    }
    hadamard = hadamard * (1.0 / (inputs as f32).sqrt());
    hadamard
}

pub const HAD32SCALE: f32 = 0.1768;
pub const HAD24SCALE: f32 = 0.2041;
pub const HAD16SCALE: f32 = 0.25;
pub const HAD12SCALE: f32 = 0.2887;
pub const HAD8SCALE: f32 = 0.3536;
pub const HAD4SCALE: f32 = 0.5;

pub fn process_hdm32(i: &[f32], o: &mut [f32]) {
    // let sf = ((1.0/(32.0f32).sqrt()));
    assert!(i.len() == 32);
    assert!(o.len() == 32);

    o[0] = i[0]
        + i[1]
        + i[2]
        + i[3]
        + i[4]
        + i[5]
        + i[6]
        + i[7]
        + i[8]
        + i[9]
        + i[10]
        + i[11]
        + i[12]
        + i[13]
        + i[14]
        + i[15]
        + i[16]
        + i[17]
        + i[18]
        + i[19]
        + i[20]
        + i[21]
        + i[22]
        + i[23]
        + i[24]
        + i[25]
        + i[26]
        + i[27]
        + i[28]
        + i[29]
        + i[30]
        + i[31];
    o[1] = i[0] - i[1] + i[2] - i[3] + i[4] - i[5] + i[6] - i[7] + i[8] - i[9] + i[10] - i[11]
        + i[12]
        - i[13]
        + i[14]
        - i[15]
        + i[16]
        - i[17]
        + i[18]
        - i[19]
        + i[20]
        - i[21]
        + i[22]
        - i[23]
        + i[24]
        - i[25]
        + i[26]
        - i[27]
        + i[28]
        - i[29]
        + i[30]
        - i[31];
    o[2] = i[0] + i[1] - i[2] - i[3] + i[4] + i[5] - i[6] - i[7] + i[8] + i[9] - i[10] - i[11]
        + i[12]
        + i[13]
        - i[14]
        - i[15]
        + i[16]
        + i[17]
        - i[18]
        - i[19]
        + i[20]
        + i[21]
        - i[22]
        - i[23]
        + i[24]
        + i[25]
        - i[26]
        - i[27]
        + i[28]
        + i[29]
        - i[30]
        - i[31];
    o[3] =
        i[0] - i[1] - i[2] + i[3] + i[4] - i[5] - i[6] + i[7] + i[8] - i[9] - i[10] + i[11] + i[12]
            - i[13]
            - i[14]
            + i[15]
            + i[16]
            - i[17]
            - i[18]
            + i[19]
            + i[20]
            - i[21]
            - i[22]
            + i[23]
            + i[24]
            - i[25]
            - i[26]
            + i[27]
            + i[28]
            - i[29]
            - i[30]
            + i[31];
    o[4] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] - i[7] + i[8] + i[9] + i[10] + i[11]
        - i[12]
        - i[13]
        - i[14]
        - i[15]
        + i[16]
        + i[17]
        + i[18]
        + i[19]
        - i[20]
        - i[21]
        - i[22]
        - i[23]
        + i[24]
        + i[25]
        + i[26]
        + i[27]
        - i[28]
        - i[29]
        - i[30]
        - i[31];
    o[5] =
        i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7] + i[8] - i[9] + i[10] - i[11] - i[12]
            + i[13]
            - i[14]
            + i[15]
            + i[16]
            - i[17]
            + i[18]
            - i[19]
            - i[20]
            + i[21]
            - i[22]
            + i[23]
            + i[24]
            - i[25]
            + i[26]
            - i[27]
            - i[28]
            + i[29]
            - i[30]
            + i[31];
    o[6] = i[0] + i[1] - i[2] - i[3] - i[4] - i[5] + i[6] + i[7] + i[8] + i[9]
        - i[10]
        - i[11]
        - i[12]
        - i[13]
        + i[14]
        + i[15]
        + i[16]
        + i[17]
        - i[18]
        - i[19]
        - i[20]
        - i[21]
        + i[22]
        + i[23]
        + i[24]
        + i[25]
        - i[26]
        - i[27]
        - i[28]
        - i[29]
        + i[30]
        + i[31];
    o[7] = i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] - i[7] + i[8] - i[9] - i[10] + i[11]
        - i[12]
        + i[13]
        + i[14]
        - i[15]
        + i[16]
        - i[17]
        - i[18]
        + i[19]
        - i[20]
        + i[21]
        + i[22]
        - i[23]
        + i[24]
        - i[25]
        - i[26]
        + i[27]
        - i[28]
        + i[29]
        + i[30]
        - i[31];
    o[8] = i[0] + i[1] + i[2] + i[3] + i[4] + i[5] + i[6] + i[7]
        - i[8]
        - i[9]
        - i[10]
        - i[11]
        - i[12]
        - i[13]
        - i[14]
        - i[15]
        + i[16]
        + i[17]
        + i[18]
        + i[19]
        + i[20]
        + i[21]
        + i[22]
        + i[23]
        - i[24]
        - i[25]
        - i[26]
        - i[27]
        - i[28]
        - i[29]
        - i[30]
        - i[31];
    o[9] = i[0] - i[1] + i[2] - i[3] + i[4] - i[5] + i[6] - i[7] - i[8] + i[9] - i[10] + i[11]
        - i[12]
        + i[13]
        - i[14]
        + i[15]
        + i[16]
        - i[17]
        + i[18]
        - i[19]
        + i[20]
        - i[21]
        + i[22]
        - i[23]
        - i[24]
        + i[25]
        - i[26]
        + i[27]
        - i[28]
        + i[29]
        - i[30]
        + i[31];
    o[10] = i[0] + i[1] - i[2] - i[3] + i[4] + i[5] - i[6] - i[7] - i[8] - i[9] + i[10] + i[11]
        - i[12]
        - i[13]
        + i[14]
        + i[15]
        + i[16]
        + i[17]
        - i[18]
        - i[19]
        + i[20]
        + i[21]
        - i[22]
        - i[23]
        - i[24]
        - i[25]
        + i[26]
        + i[27]
        - i[28]
        - i[29]
        + i[30]
        + i[31];
    o[11] =
        i[0] - i[1] - i[2] + i[3] + i[4] - i[5] - i[6] + i[7] - i[8] + i[9] + i[10] - i[11] - i[12]
            + i[13]
            + i[14]
            - i[15]
            + i[16]
            - i[17]
            - i[18]
            + i[19]
            + i[20]
            - i[21]
            - i[22]
            + i[23]
            - i[24]
            + i[25]
            + i[26]
            - i[27]
            - i[28]
            + i[29]
            + i[30]
            - i[31];
    o[12] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] - i[7] - i[8] - i[9] - i[10] - i[11]
        + i[12]
        + i[13]
        + i[14]
        + i[15]
        + i[16]
        + i[17]
        + i[18]
        + i[19]
        - i[20]
        - i[21]
        - i[22]
        - i[23]
        - i[24]
        - i[25]
        - i[26]
        - i[27]
        + i[28]
        + i[29]
        + i[30]
        + i[31];
    o[13] =
        i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7] - i[8] + i[9] - i[10] + i[11] + i[12]
            - i[13]
            + i[14]
            - i[15]
            + i[16]
            - i[17]
            + i[18]
            - i[19]
            - i[20]
            + i[21]
            - i[22]
            + i[23]
            - i[24]
            + i[25]
            - i[26]
            + i[27]
            + i[28]
            - i[29]
            + i[30]
            - i[31];
    o[14] = i[0] + i[1] - i[2] - i[3] - i[4] - i[5] + i[6] + i[7] - i[8] - i[9]
        + i[10]
        + i[11]
        + i[12]
        + i[13]
        - i[14]
        - i[15]
        + i[16]
        + i[17]
        - i[18]
        - i[19]
        - i[20]
        - i[21]
        + i[22]
        + i[23]
        - i[24]
        - i[25]
        + i[26]
        + i[27]
        + i[28]
        + i[29]
        - i[30]
        - i[31];
    o[15] = i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] - i[7] - i[8] + i[9] + i[10] - i[11]
        + i[12]
        - i[13]
        - i[14]
        + i[15]
        + i[16]
        - i[17]
        - i[18]
        + i[19]
        - i[20]
        + i[21]
        + i[22]
        - i[23]
        - i[24]
        + i[25]
        + i[26]
        - i[27]
        + i[28]
        - i[29]
        - i[30]
        + i[31];
    o[16] = i[0]
        + i[1]
        + i[2]
        + i[3]
        + i[4]
        + i[5]
        + i[6]
        + i[7]
        + i[8]
        + i[9]
        + i[10]
        + i[11]
        + i[12]
        + i[13]
        + i[14]
        + i[15]
        - i[16]
        - i[17]
        - i[18]
        - i[19]
        - i[20]
        - i[21]
        - i[22]
        - i[23]
        - i[24]
        - i[25]
        - i[26]
        - i[27]
        - i[28]
        - i[29]
        - i[30]
        - i[31];
    o[17] = i[0] - i[1] + i[2] - i[3] + i[4] - i[5] + i[6] - i[7] + i[8] - i[9] + i[10] - i[11]
        + i[12]
        - i[13]
        + i[14]
        - i[15]
        - i[16]
        + i[17]
        - i[18]
        + i[19]
        - i[20]
        + i[21]
        - i[22]
        + i[23]
        - i[24]
        + i[25]
        - i[26]
        + i[27]
        - i[28]
        + i[29]
        - i[30]
        + i[31];
    o[18] = i[0] + i[1] - i[2] - i[3] + i[4] + i[5] - i[6] - i[7] + i[8] + i[9] - i[10] - i[11]
        + i[12]
        + i[13]
        - i[14]
        - i[15]
        - i[16]
        - i[17]
        + i[18]
        + i[19]
        - i[20]
        - i[21]
        + i[22]
        + i[23]
        - i[24]
        - i[25]
        + i[26]
        + i[27]
        - i[28]
        - i[29]
        + i[30]
        + i[31];
    o[19] =
        i[0] - i[1] - i[2] + i[3] + i[4] - i[5] - i[6] + i[7] + i[8] - i[9] - i[10] + i[11] + i[12]
            - i[13]
            - i[14]
            + i[15]
            - i[16]
            + i[17]
            + i[18]
            - i[19]
            - i[20]
            + i[21]
            + i[22]
            - i[23]
            - i[24]
            + i[25]
            + i[26]
            - i[27]
            - i[28]
            + i[29]
            + i[30]
            - i[31];
    o[20] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] - i[7] + i[8] + i[9] + i[10] + i[11]
        - i[12]
        - i[13]
        - i[14]
        - i[15]
        - i[16]
        - i[17]
        - i[18]
        - i[19]
        + i[20]
        + i[21]
        + i[22]
        + i[23]
        - i[24]
        - i[25]
        - i[26]
        - i[27]
        + i[28]
        + i[29]
        + i[30]
        + i[31];
    o[21] =
        i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7] + i[8] - i[9] + i[10] - i[11] - i[12]
            + i[13]
            - i[14]
            + i[15]
            - i[16]
            + i[17]
            - i[18]
            + i[19]
            + i[20]
            - i[21]
            + i[22]
            - i[23]
            - i[24]
            + i[25]
            - i[26]
            + i[27]
            + i[28]
            - i[29]
            + i[30]
            - i[31];
    o[22] = i[0] + i[1] - i[2] - i[3] - i[4] - i[5] + i[6] + i[7] + i[8] + i[9]
        - i[10]
        - i[11]
        - i[12]
        - i[13]
        + i[14]
        + i[15]
        - i[16]
        - i[17]
        + i[18]
        + i[19]
        + i[20]
        + i[21]
        - i[22]
        - i[23]
        - i[24]
        - i[25]
        + i[26]
        + i[27]
        + i[28]
        + i[29]
        - i[30]
        - i[31];
    o[23] = i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] - i[7] + i[8] - i[9] - i[10] + i[11]
        - i[12]
        + i[13]
        + i[14]
        - i[15]
        - i[16]
        + i[17]
        + i[18]
        - i[19]
        + i[20]
        - i[21]
        - i[22]
        + i[23]
        - i[24]
        + i[25]
        + i[26]
        - i[27]
        + i[28]
        - i[29]
        - i[30]
        + i[31];
    o[24] = i[0] + i[1] + i[2] + i[3] + i[4] + i[5] + i[6] + i[7]
        - i[8]
        - i[9]
        - i[10]
        - i[11]
        - i[12]
        - i[13]
        - i[14]
        - i[15]
        - i[16]
        - i[17]
        - i[18]
        - i[19]
        - i[20]
        - i[21]
        - i[22]
        - i[23]
        + i[24]
        + i[25]
        + i[26]
        + i[27]
        + i[28]
        + i[29]
        + i[30]
        + i[31];
    o[25] = i[0] - i[1] + i[2] - i[3] + i[4] - i[5] + i[6] - i[7] - i[8] + i[9] - i[10] + i[11]
        - i[12]
        + i[13]
        - i[14]
        + i[15]
        - i[16]
        + i[17]
        - i[18]
        + i[19]
        - i[20]
        + i[21]
        - i[22]
        + i[23]
        + i[24]
        - i[25]
        + i[26]
        - i[27]
        + i[28]
        - i[29]
        + i[30]
        - i[31];
    o[26] = i[0] + i[1] - i[2] - i[3] + i[4] + i[5] - i[6] - i[7] - i[8] - i[9] + i[10] + i[11]
        - i[12]
        - i[13]
        + i[14]
        + i[15]
        - i[16]
        - i[17]
        + i[18]
        + i[19]
        - i[20]
        - i[21]
        + i[22]
        + i[23]
        + i[24]
        + i[25]
        - i[26]
        - i[27]
        + i[28]
        + i[29]
        - i[30]
        - i[31];
    o[27] =
        i[0] - i[1] - i[2] + i[3] + i[4] - i[5] - i[6] + i[7] - i[8] + i[9] + i[10] - i[11] - i[12]
            + i[13]
            + i[14]
            - i[15]
            - i[16]
            + i[17]
            + i[18]
            - i[19]
            - i[20]
            + i[21]
            + i[22]
            - i[23]
            + i[24]
            - i[25]
            - i[26]
            + i[27]
            + i[28]
            - i[29]
            - i[30]
            + i[31];
    o[28] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] - i[7] - i[8] - i[9] - i[10] - i[11]
        + i[12]
        + i[13]
        + i[14]
        + i[15]
        - i[16]
        - i[17]
        - i[18]
        - i[19]
        + i[20]
        + i[21]
        + i[22]
        + i[23]
        + i[24]
        + i[25]
        + i[26]
        + i[27]
        - i[28]
        - i[29]
        - i[30]
        - i[31];
    o[29] =
        i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7] - i[8] + i[9] - i[10] + i[11] + i[12]
            - i[13]
            + i[14]
            - i[15]
            - i[16]
            + i[17]
            - i[18]
            + i[19]
            + i[20]
            - i[21]
            + i[22]
            - i[23]
            + i[24]
            - i[25]
            + i[26]
            - i[27]
            - i[28]
            + i[29]
            - i[30]
            + i[31];
    o[30] = i[0] + i[1] - i[2] - i[3] - i[4] - i[5] + i[6] + i[7] - i[8] - i[9]
        + i[10]
        + i[11]
        + i[12]
        + i[13]
        - i[14]
        - i[15]
        - i[16]
        - i[17]
        + i[18]
        + i[19]
        + i[20]
        + i[21]
        - i[22]
        - i[23]
        + i[24]
        + i[25]
        - i[26]
        - i[27]
        - i[28]
        - i[29]
        + i[30]
        + i[31];
    o[31] = i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] - i[7] - i[8] + i[9] + i[10] - i[11]
        + i[12]
        - i[13]
        - i[14]
        + i[15]
        - i[16]
        + i[17]
        + i[18]
        - i[19]
        + i[20]
        - i[21]
        - i[22]
        + i[23]
        + i[24]
        - i[25]
        - i[26]
        + i[27]
        - i[28]
        + i[29]
        + i[30]
        - i[31];
    for s in o.iter_mut() {
        *s *= HAD32SCALE;
    }
}

pub fn process_hdm24<T: Float>(i: &[T], o: &mut [T]) {
    assert!(i.len() == 24);
    assert!(o.len() == 24);

    o[0] = i[0]
        + i[1]
        + i[2]
        + i[3]
        + i[4]
        + i[5]
        + i[6]
        + i[7]
        + i[8]
        + i[9]
        + i[10]
        + i[11]
        + i[12]
        + i[13]
        + i[14]
        + i[15]
        + i[16]
        + i[17]
        + i[18]
        + i[19]
        + i[20]
        + i[21]
        + i[22]
        + i[23];
    o[1] = i[0] - i[1] + i[2] - i[3] + i[4] + i[5] + i[6] - i[7] - i[8] - i[9] + i[10] - i[11]
        + i[12]
        - i[13]
        + i[14]
        - i[15]
        + i[16]
        + i[17]
        + i[18]
        - i[19]
        - i[20]
        - i[21]
        + i[22]
        - i[23];
    o[2] =
        i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] + i[7] - i[8] - i[9] - i[10] + i[11] + i[12]
            - i[13]
            - i[14]
            + i[15]
            - i[16]
            + i[17]
            + i[18]
            + i[19]
            - i[20]
            - i[21]
            - i[22]
            + i[23];
    o[3] = i[0] + i[1] - i[2] - i[3] + i[4] - i[5] + i[6] + i[7] + i[8] - i[9] - i[10] - i[11]
        + i[12]
        + i[13]
        - i[14]
        - i[15]
        + i[16]
        - i[17]
        + i[18]
        + i[19]
        + i[20]
        - i[21]
        - i[22]
        - i[23];
    o[4] = i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7] + i[8] + i[9] - i[10] - i[11]
        + i[12]
        - i[13]
        + i[14]
        - i[15]
        - i[16]
        + i[17]
        - i[18]
        + i[19]
        + i[20]
        + i[21]
        - i[22]
        - i[23];
    o[5] = i[0] - i[1] - i[2] + i[3] - i[4] - i[5] + i[6] - i[7] + i[8] + i[9] + i[10] - i[11]
        + i[12]
        - i[13]
        - i[14]
        + i[15]
        - i[16]
        - i[17]
        + i[18]
        - i[19]
        + i[20]
        + i[21]
        + i[22]
        - i[23];
    o[6] =
        i[0] - i[1] - i[2] - i[3] + i[4] - i[5] - i[6] + i[7] - i[8] + i[9] + i[10] + i[11] + i[12]
            - i[13]
            - i[14]
            - i[15]
            + i[16]
            - i[17]
            - i[18]
            + i[19]
            - i[20]
            + i[21]
            + i[22]
            + i[23];
    o[7] = i[0] + i[1] - i[2] - i[3] - i[4] + i[5] - i[6] - i[7] + i[8] - i[9]
        + i[10]
        + i[11]
        + i[12]
        + i[13]
        - i[14]
        - i[15]
        - i[16]
        + i[17]
        - i[18]
        - i[19]
        + i[20]
        - i[21]
        + i[22]
        + i[23];
    o[8] = i[0] + i[1] + i[2] - i[3] - i[4] - i[5] + i[6] - i[7] - i[8] + i[9] - i[10]
        + i[11]
        + i[12]
        + i[13]
        + i[14]
        - i[15]
        - i[16]
        - i[17]
        + i[18]
        - i[19]
        - i[20]
        + i[21]
        - i[22]
        + i[23];
    o[9] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] + i[7] - i[8] - i[9] + i[10] - i[11]
        + i[12]
        + i[13]
        + i[14]
        + i[15]
        - i[16]
        - i[17]
        - i[18]
        + i[19]
        - i[20]
        - i[21]
        + i[22]
        - i[23];
    o[10] =
        i[0] - i[1] + i[2] + i[3] + i[4] - i[5] - i[6] - i[7] + i[8] - i[9] - i[10] + i[11] + i[12]
            - i[13]
            + i[14]
            + i[15]
            + i[16]
            - i[17]
            - i[18]
            - i[19]
            + i[20]
            - i[21]
            - i[22]
            + i[23];
    o[11] = i[0] + i[1] - i[2] + i[3] + i[4] + i[5] - i[6] - i[7] - i[8] + i[9] - i[10] - i[11]
        + i[12]
        + i[13]
        - i[14]
        + i[15]
        + i[16]
        + i[17]
        - i[18]
        - i[19]
        - i[20]
        + i[21]
        - i[22]
        - i[23];
    o[12] = i[0] + i[1] + i[2] + i[3] + i[4] + i[5] + i[6] + i[7] + i[8] + i[9] + i[10] + i[11]
        - i[12]
        - i[13]
        - i[14]
        - i[15]
        - i[16]
        - i[17]
        - i[18]
        - i[19]
        - i[20]
        - i[21]
        - i[22]
        - i[23];
    o[13] =
        i[0] - i[1] + i[2] - i[3] + i[4] + i[5] + i[6] - i[7] - i[8] - i[9] + i[10] - i[11] - i[12]
            + i[13]
            - i[14]
            + i[15]
            - i[16]
            - i[17]
            - i[18]
            + i[19]
            + i[20]
            + i[21]
            - i[22]
            + i[23];
    o[14] = i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] + i[7] - i[8] - i[9] - i[10] + i[11]
        - i[12]
        + i[13]
        + i[14]
        - i[15]
        + i[16]
        - i[17]
        - i[18]
        - i[19]
        + i[20]
        + i[21]
        + i[22]
        - i[23];
    o[15] = i[0] + i[1] - i[2] - i[3] + i[4] - i[5] + i[6] + i[7] + i[8]
        - i[9]
        - i[10]
        - i[11]
        - i[12]
        - i[13]
        + i[14]
        + i[15]
        - i[16]
        + i[17]
        - i[18]
        - i[19]
        - i[20]
        + i[21]
        + i[22]
        + i[23];
    o[16] =
        i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7] + i[8] + i[9] - i[10] - i[11] - i[12]
            + i[13]
            - i[14]
            + i[15]
            + i[16]
            - i[17]
            + i[18]
            - i[19]
            - i[20]
            - i[21]
            + i[22]
            + i[23];
    o[17] =
        i[0] - i[1] - i[2] + i[3] - i[4] - i[5] + i[6] - i[7] + i[8] + i[9] + i[10] - i[11] - i[12]
            + i[13]
            + i[14]
            - i[15]
            + i[16]
            + i[17]
            - i[18]
            + i[19]
            - i[20]
            - i[21]
            - i[22]
            + i[23];
    o[18] = i[0] - i[1] - i[2] - i[3] + i[4] - i[5] - i[6] + i[7] - i[8] + i[9] + i[10] + i[11]
        - i[12]
        + i[13]
        + i[14]
        + i[15]
        - i[16]
        + i[17]
        + i[18]
        - i[19]
        + i[20]
        - i[21]
        - i[22]
        - i[23];
    o[19] = i[0] + i[1] - i[2] - i[3] - i[4] + i[5] - i[6] - i[7] + i[8] - i[9] + i[10] + i[11]
        - i[12]
        - i[13]
        + i[14]
        + i[15]
        + i[16]
        - i[17]
        + i[18]
        + i[19]
        - i[20]
        + i[21]
        - i[22]
        - i[23];
    o[20] = i[0] + i[1] + i[2] - i[3] - i[4] - i[5] + i[6] - i[7] - i[8] + i[9] - i[10] + i[11]
        - i[12]
        - i[13]
        - i[14]
        + i[15]
        + i[16]
        + i[17]
        - i[18]
        + i[19]
        + i[20]
        - i[21]
        + i[22]
        - i[23];
    o[21] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] + i[7] - i[8] - i[9] + i[10]
        - i[11]
        - i[12]
        - i[13]
        - i[14]
        - i[15]
        + i[16]
        + i[17]
        + i[18]
        - i[19]
        + i[20]
        + i[21]
        - i[22]
        + i[23];
    o[22] = i[0] - i[1] + i[2] + i[3] + i[4] - i[5] - i[6] - i[7] + i[8] - i[9] - i[10] + i[11]
        - i[12]
        + i[13]
        - i[14]
        - i[15]
        - i[16]
        + i[17]
        + i[18]
        + i[19]
        - i[20]
        + i[21]
        + i[22]
        - i[23];
    o[23] = i[0] + i[1] - i[2] + i[3] + i[4] + i[5] - i[6] - i[7] - i[8] + i[9]
        - i[10]
        - i[11]
        - i[12]
        - i[13]
        + i[14]
        - i[15]
        - i[16]
        - i[17]
        + i[18]
        + i[19]
        + i[20]
        - i[21]
        + i[22]
        + i[23];
    for s in o.iter_mut() {
        *s = *s * T::from(HAD24SCALE).unwrap(); 
    }
}

pub fn process_hdm16(i: &[f32], o: &mut [f32]) {
    assert!(i.len() == 16);
    assert!(o.len() == 16);
    o[0] = i[0]
        + i[1]
        + i[2]
        + i[3]
        + i[4]
        + i[5]
        + i[6]
        + i[7]
        + i[8]
        + i[9]
        + i[10]
        + i[11]
        + i[12]
        + i[13]
        + i[14]
        + i[15];
    o[1] = i[0] - i[1] + i[2] - i[3] + i[4] - i[5] + i[6] - i[7] + i[8] - i[9] + i[10] - i[11]
        + i[12]
        - i[13]
        + i[14]
        - i[15];
    o[2] = i[0] + i[1] - i[2] - i[3] + i[4] + i[5] - i[6] - i[7] + i[8] + i[9] - i[10] - i[11]
        + i[12]
        + i[13]
        - i[14]
        - i[15];
    o[3] =
        i[0] - i[1] - i[2] + i[3] + i[4] - i[5] - i[6] + i[7] + i[8] - i[9] - i[10] + i[11] + i[12]
            - i[13]
            - i[14]
            + i[15];
    o[4] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] - i[7] + i[8] + i[9] + i[10] + i[11]
        - i[12]
        - i[13]
        - i[14]
        - i[15];
    o[5] =
        i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7] + i[8] - i[9] + i[10] - i[11] - i[12]
            + i[13]
            - i[14]
            + i[15];
    o[6] = i[0] + i[1] - i[2] - i[3] - i[4] - i[5] + i[6] + i[7] + i[8] + i[9]
        - i[10]
        - i[11]
        - i[12]
        - i[13]
        + i[14]
        + i[15];
    o[7] = i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] - i[7] + i[8] - i[9] - i[10] + i[11]
        - i[12]
        + i[13]
        + i[14]
        - i[15];
    o[8] = i[0] + i[1] + i[2] + i[3] + i[4] + i[5] + i[6] + i[7]
        - i[8]
        - i[9]
        - i[10]
        - i[11]
        - i[12]
        - i[13]
        - i[14]
        - i[15];
    o[9] = i[0] - i[1] + i[2] - i[3] + i[4] - i[5] + i[6] - i[7] - i[8] + i[9] - i[10] + i[11]
        - i[12]
        + i[13]
        - i[14]
        + i[15];
    o[10] = i[0] + i[1] - i[2] - i[3] + i[4] + i[5] - i[6] - i[7] - i[8] - i[9] + i[10] + i[11]
        - i[12]
        - i[13]
        + i[14]
        + i[15];
    o[11] =
        i[0] - i[1] - i[2] + i[3] + i[4] - i[5] - i[6] + i[7] - i[8] + i[9] + i[10] - i[11] - i[12]
            + i[13]
            + i[14]
            - i[15];
    o[12] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] - i[7] - i[8] - i[9] - i[10] - i[11]
        + i[12]
        + i[13]
        + i[14]
        + i[15];
    o[13] =
        i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7] - i[8] + i[9] - i[10] + i[11] + i[12]
            - i[13]
            + i[14]
            - i[15];
    o[14] = i[0] + i[1] - i[2] - i[3] - i[4] - i[5] + i[6] + i[7] - i[8] - i[9]
        + i[10]
        + i[11]
        + i[12]
        + i[13]
        - i[14]
        - i[15];
    o[15] = i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] - i[7] - i[8] + i[9] + i[10] - i[11]
        + i[12]
        - i[13]
        - i[14]
        + i[15];
    for s in o.iter_mut() {
        *s *= HAD16SCALE;
    }
}
pub fn process_hdm12(i: &[f32], o: &mut [f32; 12]) {
    assert!(i.len() == 12);
    assert!(o.len() == 12);
    o[0] = i[0] + i[1] + i[2] + i[3] + i[4] + i[5] + i[6] + i[7] + i[8] + i[9] + i[10] + i[11];
    o[1] = i[0] - i[1] + i[2] - i[3] + i[4] + i[5] + i[6] - i[7] - i[8] - i[9] + i[10] - i[11];
    o[2] = i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] + i[7] - i[8] - i[9] - i[10] + i[11];
    o[3] = i[0] + i[1] - i[2] - i[3] + i[4] - i[5] + i[6] + i[7] + i[8] - i[9] - i[10] - i[11];
    o[4] = i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7] + i[8] + i[9] - i[10] - i[11];
    o[5] = i[0] - i[1] - i[2] + i[3] - i[4] - i[5] + i[6] - i[7] + i[8] + i[9] + i[10] - i[11];
    o[6] = i[0] - i[1] - i[2] - i[3] + i[4] - i[5] - i[6] + i[7] - i[8] + i[9] + i[10] + i[11];
    o[7] = i[0] + i[1] - i[2] - i[3] - i[4] + i[5] - i[6] - i[7] + i[8] - i[9] + i[10] + i[11];
    o[8] = i[0] + i[1] + i[2] - i[3] - i[4] - i[5] + i[6] - i[7] - i[8] + i[9] - i[10] + i[11];
    o[9] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] + i[7] - i[8] - i[9] + i[10] - i[11];
    o[10] = i[0] - i[1] + i[2] + i[3] + i[4] - i[5] - i[6] - i[7] + i[8] - i[9] - i[10] + i[11];
    o[11] = i[0] + i[1] - i[2] + i[3] + i[4] + i[5] - i[6] - i[7] - i[8] + i[9] - i[10] - i[11];
    for s in o.iter_mut() {
        *s *= HAD12SCALE;
    }
}
pub fn process_hdm8(i: &[f32], o: &mut [f32]) {
    assert!(i.len() == 8);
    assert!(o.len() == 8);
    o[0] = i[0] + i[1] + i[2] + i[3] + i[4] + i[5] + i[6] + i[7];
    o[1] = i[0] - i[1] + i[2] - i[3] + i[4] - i[5] + i[6] - i[7];
    o[2] = i[0] + i[1] - i[2] - i[3] + i[4] + i[5] - i[6] - i[7];
    o[3] = i[0] - i[1] - i[2] + i[3] + i[4] - i[5] - i[6] + i[7];
    o[4] = i[0] + i[1] + i[2] + i[3] - i[4] - i[5] - i[6] - i[7];
    o[5] = i[0] - i[1] + i[2] - i[3] - i[4] + i[5] - i[6] + i[7];
    o[6] = i[0] + i[1] - i[2] - i[3] - i[4] - i[5] + i[6] + i[7];
    o[7] = i[0] - i[1] - i[2] + i[3] - i[4] + i[5] + i[6] - i[7];

    for s in o.iter_mut() {
        *s *= HAD8SCALE;
    }
}

pub fn process_hdm4(i: &[f32], o: &mut [f32]) {
    assert!(i.len() == 4);
    assert!(o.len() == 4);
    o[0] = i[0] + i[1] + i[2] + i[3];
    o[1] = i[0] - i[1] + i[2] - i[3];
    o[2] = i[0] + i[1] - i[2] - i[3];
    o[3] = i[0] - i[1] - i[2] + i[3];

    for s in o.iter_mut() {
        *s *= HAD4SCALE;
    }
}
