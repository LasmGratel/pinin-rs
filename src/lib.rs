pub mod dict_loader;
mod pinin;
mod compressed;
mod elements;
mod keyboard;
mod format;
mod searcher;
mod accelerator;
mod cache;

#[cfg(test)]
mod tests {
    use crate::pinin::PinIn;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);

        let pinin = PinIn::new();


    }
}


