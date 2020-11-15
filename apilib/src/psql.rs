


pub fn query(connection_string : &str) {
    use pq_sys::*;

    unsafe {
        let mut conn = PQconnectdb("host=localhost user=wojciech dbname=rusty_games password=password".as_bytes().as_ptr() as *const i8);

        println!("{:?}", PQstatus(conn));



        PQfinish(conn);
    }

}



