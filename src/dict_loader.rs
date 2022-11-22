pub fn load_dict<F>(action: F) where F: Fn(char, Vec<&str>) {
    let dict_str = include_str!("dict.txt");
    dict_str.lines().for_each(|line| {
        let ch = line.chars().next().unwrap();
        let records = line[3..].split(", ").collect::<Vec<&str>>();
        action(ch, records);
    });
}