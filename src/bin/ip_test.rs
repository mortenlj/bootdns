fn main() {
    let ifaces = if_addrs::get_if_addrs().unwrap();
    println!("Got list of interfaces");
    println!("{:#?}", ifaces);
}
