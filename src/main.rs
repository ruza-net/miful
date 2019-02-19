mod parsing;
mod driver;

fn main() {
    let test =
    "[
        print 3
    ]";

    let mut driver = driver::Driver::new(test);

    driver.process();
}
