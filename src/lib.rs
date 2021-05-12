use json::JsonValue;
use reqwest::Response;
use std::collections::HashMap;

pub struct LoginConfig {
    username: String,
    passwd: String,
}
pub struct User {
    pub name: String,
    pub login: String,
    pub school: String,
    pub course: Vec<Course>,
}
pub struct Course {
    pub course_id: String,
    pub course_name: String,
    pub teacher: String,
    pub sign_info: Vec<SignInfo>,
}
pub struct SignInfo {
    pub time: String,
    pub code: String,
    pub attendance_id: String,
    pub attendance_mode: String,
    pub statue: String,
}

pub enum Api {
    LoginApi(String),
    SignPostApi(String),
    CourseApi(String),
    SignAttentionApi(String),
}

impl LoginConfig {
    pub fn new(args: &[String]) -> Result<LoginConfig, &'static str> {
        if args.len() < 3 {
            return Err("no enough.");
        }
        let username = args[1].clone();
        let passwd = args[2].clone();
        Ok(LoginConfig { username, passwd })
    }
    pub fn login(
        self,
        api: Api,
        client: &reqwest::blocking::Client,
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let mut form = HashMap::new();
        form.insert("login", self.username);
        form.insert("password", self.passwd);
        // let client = reqwest::blocking::Client::builder().cookie_store(true).user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36").cookie_store(true).build().expect("Client build Error.");
        // let client = reqwest::Client::builder().user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36").cookie_store(true).build().expect("Client build Error.");

        let res = client
            .post(if let Api::LoginApi(url) = api {
                url
            } else {
                panic!("Api type Error.")
            })
            .json(&form)
            .send();

        res
    }
}

pub fn get_course(login: String, api: Api, client: &reqwest::blocking::Client) -> Vec<Course> {
    let mut form = HashMap::new();
    form.insert("category", "undefined");
    form.insert("status", "undefined");
    form.insert("page", "1");
    form.insert("per_page", "16");
    form.insert("sort_by", "updated_at");
    form.insert("sort_direction", "desc");
    form.insert("username", &login);
    let res = client
        .get(if let Api::CourseApi(url) = api {
            url
        } else {
            panic!("Error in User::new().")
        })
        .json(&form)
        .send()
        .expect("Error: client.get.json.send.");
    let js = json::parse(&res.text().unwrap()).expect("Error: get_course::json::parse.");
    let mut courses: Vec<Course> = vec![];
    for course in js["courses"].members() {
        let sign_attention_api = format!("https://data.educoder.net/api/courses/{id}/attendances.json?coursesId={id}&id={id}&status={status}&page=1", id = course["id"], status = "all");
        let sign_info: Vec<SignInfo> =
            get_sign_info(Api::SignAttentionApi(sign_attention_api), &client);
        courses.push(Course::new(
            course["id"].to_string(),
            course["name"].to_string(),
            course["teacher"]["real_name"].to_string(),
            sign_info,
        ))
    }
    // println!("{}", js["courses"][0]);
    courses
}

pub fn get_sign_info(api: Api, client: &reqwest::blocking::Client) -> Vec<SignInfo> {
    let res = client
        .get(if let Api::SignAttentionApi(url) = api {
            url
        } else {
            panic!("Error in get_sign_info() client.get(url)")
        })
        .send()
        .expect("Error in get_sign_info() client.send()");
    let js = json::parse(&res.text().unwrap()).expect("Error in get_sign_info() json::parse()");

    let mut sign_infos: Vec<SignInfo> = vec![];
    for sign in js["attendances"].members() {
        // println!("{:?}", sign["id"].to_string());
        sign_infos.push(SignInfo::new(
            sign["attendance_date"].to_string(),
            sign["attendance_code"].to_string(),
            sign["id"].to_string(),
            sign["mode"].to_string(),
            sign["status"].to_string(),
        ));
    }

    sign_infos
}
impl User {
    pub fn new(name: String, login: String, school: String, course: Vec<Course>) -> User {
        User {
            name,
            login,
            school,
            course,
        }
    }
    pub fn get_user(js: &JsonValue, client: &reqwest::blocking::Client) -> User {
        let get_course_api = format!(
            "https://data.educoder.net/api/users/{}/courses.json",
            js["login"]
        );
        let courses: Vec<Course> = get_course(
            js["login"].to_string(),
            Api::CourseApi(get_course_api),
            &client,
        );
        let user = User::new(
            js["name"].to_string(),
            js["login"].to_string(),
            js["school"].to_string(),
            courses,
        );
        user
    }
    pub fn print_course(&self) {
        println!("index    id      name      teacher");
        let mut i = 1;
        for course in &self.course {
            println!(
                "{}      {}    {}     {}",
                i, course.course_id, course.course_name, course.teacher
            );
        }
    }
}

impl Course {
    pub fn new(
        course_id: String,
        course_name: String,
        teacher: String,
        sign_info: Vec<SignInfo>,
    ) -> Course {
        Course {
            course_id,
            course_name,
            teacher,
            sign_info,
        }
    }
    pub fn print_sign_code(&self) {
        println!("index        date    code    status");
        let mut i = 0;
        for signItem in &self.sign_info {
            println!(
                "{}      {}    {}    {}",
                i, signItem.time, signItem.code, signItem.statue
            );
            i += 1;
        }
    }
    pub fn get_sign_code(mut self, client: &reqwest::blocking::Client) {
        let sign_attention_api = format!("https://data.educoder.net/api/courses/{id}/attendances.json?coursesId={id}&id={id}&status={status}&page=1", id = self.course_id, status = "all");
        let sign = get_sign_info(Api::SignAttentionApi(sign_attention_api), client);
        self.sign_info = sign;
    }
}

impl SignInfo {
    pub fn new(
        time: String,
        code: String,
        attendance_id: String,
        attendance_mode: String,
        statue: String,
    ) -> SignInfo {
        SignInfo {
            time,
            code,
            attendance_id,
            attendance_mode,
            statue,
        }
    }
    pub fn sign_post(
        &self,
        api: Api,
        client: &reqwest::blocking::Client,
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        // creat form
        let mut form = HashMap::new();
        form.insert("attendance_id".to_string(), &self.attendance_id);
        form.insert("attendance_mode".to_string(), &self.attendance_mode);
        form.insert("code".to_string(), &self.code);

        // let client = reqwest::blocking::Client::new();
        client
            .post(if let Api::SignPostApi(url) = api {
                url
            } else {
                panic!("Api type Error: expect SignPostApi.")
            })
            .json(&form)
            .send()
    }
}
