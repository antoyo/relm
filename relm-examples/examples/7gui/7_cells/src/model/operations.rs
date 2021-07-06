/// Apply the operation with the given name to the values.
pub fn operate(function: &str, values: Vec<f64>) -> Result<f64, String> {
    match function {
        "add" => add(values),
        "sub" => sub(values),
        "div" => div(values),
        "mul" => mul(values),
        "sum" => sum(values),
        "prod" => prod(values),
        _ => Err(format!("Unknown function: {}", function)),
    }
}

fn add(values: Vec<f64>) -> Result<f64, String> {
    if values.len() != 2 {
        Err("add takes exactly two arguments".to_string())
    } else {
        Ok(values.iter().sum())
    }
}

fn sub(values: Vec<f64>) -> Result<f64, String> {
    if values.len() != 2 {
        Err("sub takes exactly two arguments".to_string())
    } else {
        Ok(values[0] - values[1])
    }
}

fn div(values: Vec<f64>) -> Result<f64, String> {
    if values.len() != 2 {
        Err("div takes exactly two arguments".to_string())
    } else if values[1] == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(values[0] / values[1])
    }
}

fn mul(values: Vec<f64>) -> Result<f64, String> {
    if values.len() != 2 {
        Err("mul takes exactly two arguments".to_string())
    } else {
        Ok(values[0] * values[1])
    }
}

fn sum(values: Vec<f64>) -> Result<f64, String> {
    Ok(values.iter().sum())
}

fn prod(values: Vec<f64>) -> Result<f64, String> {
    Ok(values.iter().product())
}
