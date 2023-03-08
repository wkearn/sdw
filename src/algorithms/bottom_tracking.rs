//! Statistical bottom tracking

/// Track the bottom by fitting a Bayesian changepoint model
///
/// The sonar data is supplied as a `Vec<u16>` and the return value is a
/// `Vec<f64>` with the posterior probabilities of the bottom within each
/// range bin.
pub fn bottom_track(data: Vec<u16>) -> Vec<f64> {
    let n: i32 = data
        .len()
        .try_into()
        .expect("Vector size must be less than 2147483647");

    // Precompute the sum of the data
    let ys: u32 = data.iter().map(|x| *x as u32).sum();

    // Precompute the cumulative sum of the data
    let yc: Vec<u32> = data
        .iter()
        .scan(0u32, |acc, &x| {
            *acc += x as u32;
            Some(*acc)
        })
        .collect();

    // Prior is a geometric distribution on 1..n
    let prior: Vec<f64> = (1..n)
        .map(|k| (0.1f64).ln() + f64::from(k - 1) * (0.9f64).ln())
        .collect();
    let w0: Vec<f64> = prior.iter().map(|x| x.exp()).collect();

    let theta = maximize(w0.as_slice(), yc.as_slice(), ys, n);

    let iterations = 10;
    
    let theta2 = (1..iterations).fold(theta,|acc,_| {
	em_step(acc,prior.as_slice(),yc.as_slice(),ys,n)
    });
    
    expectation(theta2,prior.as_slice(),yc.as_slice(),ys,n)
}

fn em_step(theta: (f64,f64), p0: &[f64], yc: &[u32], ys: u32, n: i32) -> (f64,f64) {
    let w = expectation(theta,p0,yc,ys,n);
    maximize(w.as_slice(),yc,ys,n)
}

fn expectation(theta: (f64, f64), p0: &[f64], yc: &[u32], ys: u32, n: i32) -> Vec<f64> {
    let (lambda1, lambda2) = theta;

    let v: Vec<f64> = yc.iter().zip(p0.iter()).zip(1..n).map(|((&y, p), tau)| {
        lambda1.ln() * f64::from(tau)
            + lambda2.ln() * f64::from(n - tau)
            + (lambda2 - lambda1) * f64::from(y)
            + lambda2 * f64::from(ys)
            + p
    }).collect();

    softmax(v.as_slice())
}

fn maximize(w: &[f64], yc: &[u32], ys: u32, n: i32) -> (f64, f64) {
    // Compute the expected value of tau
    let etau0: f64 = w.iter().zip(0..).map(|(x, i)| x * f64::from(i)).sum();
    // Compute the expected value of the sum of y[1..tau]
    let eyc: f64 = w
        .iter()
        .zip(yc.iter())
        .map(|(w, y)| w * f64::from(*y))
        .sum();

    let lambda1 = etau0 / eyc;
    let lambda2 = (f64::from(n) - etau0) / (f64::from(ys) - eyc);
    (lambda1, lambda2)
}

// Streaming logsumexp after Sebastian Nowozin
fn logsumexp_stream(x: &[f64]) -> f64 {
    let (alpha, r) = x.iter().fold((f64::NEG_INFINITY, 0.0), |state, &x| {
        let (mut alpha, mut r) = state;
        if x <= alpha {
            r += (x - alpha).exp()
        } else {
            r *= (alpha - x).exp();
            r += 1.0;
            alpha = x;
        }
        (alpha, r)
    });
    r.ln() + alpha
}

fn softmax(x: &[f64]) -> Vec<f64> {
    let d = logsumexp_stream(x);

    x.iter().map(|x| (x - d).exp()).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_logsumexp() {
        let v = &[0.0; 100];
        let d = logsumexp(v);

        assert_eq!(d, (100.0f64).ln());

        let v2 = &[0.0, 0.0, 1000.0];
        let d = logsumexp(v2);

        assert_eq!(d, 1000.0);
    }

    #[test]
    fn test_logsumexp_stream() {
        let v = &[0.0; 100];
        let d = logsumexp_stream(v);
        assert_eq!(d, (100.0f64).ln());

        let v2 = &[0.0, 0.0, 1000.0];
        let d = logsumexp_stream(v2);
        assert_eq!(d, 1000.0);
    }

    #[test]
    fn test_softmax() {
        let v1 = &[0.0, 0.0, 1000.0];
        assert_eq!(softmax(v1), vec![0.0, 0.0, 1.0]);

        let v2 = &[0.0, 0.0, 0.0, 0.0];
        assert_eq!(softmax(v2), vec![0.25, 0.25, 0.25, 0.25]);
    }
}
