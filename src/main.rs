mod parsing;
mod driver;

fn main() {
    let test =
    "[
        define foo {
            [
                if [> [first (1 2 3)] [last (1 2 3)]] {
                    [print [string (Hello world!)]]
                } {
                    [print No]
                }
            ]
        }
    ]";

    let mut driver = driver::Driver::new(test);

    driver.process();
}
