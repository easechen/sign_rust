use std::thread::sleep;
use sign::{User, Api, get_course, get_sign_info, Course};
use std::process::exit;
use std::io::{stdin, stdout, Write};
use std::env;
fn main() {
    let client = reqwest::blocking::Client::builder().cookie_store(true).user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36").cookie_store(true).build().expect("Client build Error.");
    let args: Vec<String> = env::args().collect();

    let login_config = sign::LoginConfig::new(&args).unwrap_or_else(|err| {
        println!("{}", err);
        exit(1);
    });
    let res = login_config.login(Api::LoginApi(String::from("https://data.educoder.net/api/accounts/login.json")), &client).unwrap().text().unwrap();
    // println!("{}", res);
    // if login error
    if !res.contains("name") {
        let js = json::parse(&res).unwrap();
        eprintln!("{}", js["message"]);
        exit(-1);
    }
    let js = json::parse(&res).unwrap();
    // let get_course_api = format!("https://data.educoder.net/api/users/{}/courses.json", js["login"]);
    // let courses: Vec<Course> = get_course(js["login"].to_string(), Api::CourseApi(get_course_api), &client);
    // let user = User::new(js["name"].to_string(), js["login"].to_string(), js["school"].to_string(), courses);
    let user = User::get_user(&js, &client);

    //print welcome and courses
    println!("你好，来自 {} 的 {} !", user.school, user.name);
    user.print_course();
    //select course in courses
    let mut course_select = String::new();
    stdin().read_line(&mut course_select).expect("Stdin::read_line Error");
    let index: usize = course_select.trim().parse().expect("Please enter a number.");
    let course: &Course = user.course.get(index - 1).expect("Error: Type right index.");
    loop {
        println!("1,sign code 2,sign 3,quit");
        let mut psc = String::new(); stdin().read_line(&mut psc).expect("Error loop stdin().read_line().");
        let index:usize = psc.trim().parse().expect("Please enter a number");
        match index {
            1  => {
                course.print_sign_code();
            },
            2 => {
                print!("waiting...");
                // course.get_sign_code(&client);
                // while course.sign_info.get(0).unwrap().statue == "history" {
                //     // course_selected: &Course = &user.course.get(index-1).expect("Error: Type right index.");
                //     // println!("{}", course.sign_info.get(0).unwrap().statue);
                //     // let sign = &mut course.get_sign_code(&client);
                //     print!("."); stdout().flush().unwrap();
                //     sleep(Duration::from_secs(1));
                // }
                if course.sign_info.get(0).unwrap().statue != "history" {
                    let res = course.sign_info.get(0).unwrap().sign_post(Api::LoginApi(String::from("https://data.educoder.net/api/weapps/course_member_attendances.json")), &client).expect("login error");
                    if res.text().unwrap().contains("success") {
                        println!("success, code is {}", course.sign_info.get(0).unwrap().code);
                        break;
                    }
                } else {
                    println!("no sign right now.");
                }
            }
            3 => {
                println!("have exited.");
                break;
            }
            _ => {println!("Type right chiose.");}
        }
    }


    // get_sign_info(Api::SignAttentionApi(String::from("https://data.educoder.net/api/courses/10779/attendances.json?coursesId=10779&id=10779&status=all&page=1")), &client);
}
