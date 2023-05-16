use std::str::FromStr;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
}

#[non_exhaustive]
#[derive(Debug)]
pub enum Sentence {
}

impl FromStr for Sentence {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> { todo!() }
}

#[cfg(test)]
mod tests {
    use super::*;
}
