use std::mem::swap;

/// Iterate on (one element, rest of collection) pairs.
pub fn one_rest_split_iter<T, F>(mut vec: &mut Vec<T>, mut f: F) where F: FnMut(&mut T, &mut Vec<T>) {
    let mut kept = vec.remove(0);
    f(&mut kept, &mut vec);
    for i in 0..vec.len() {
        swap(&mut vec[i], &mut kept);
        f(&mut kept, &mut vec);
    }
    vec.push(kept);
}



#[cfg(test)]
mod tests {
    use std::fmt;

    struct NonTrivialThing {
        i: i32,
    }

    impl NonTrivialThing {
        fn smthg(&mut self, _r: &mut Vec<NonTrivialThing>) -> String {
            format!("{}", self.i)
        }
    }

    impl fmt::Debug for NonTrivialThing {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.i)
        }
    }

    #[test]
    fn test_iter() {
        fn thing(i: i32) -> NonTrivialThing {
            NonTrivialThing { i: i }
        }

        fn main() {
            let mut v = vec![thing(0), thing(1), thing(2), thing(3)];
            println!("{:?}", v);
            one_rest_split_iter(&mut v, |i, r| {
                println!("{} {:?}", i.smthg(r), r);
            });
        }
    }
}
