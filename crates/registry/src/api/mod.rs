use actix_web::web;

pub mod identity;
pub mod log;
pub mod package;
pub mod scope;
pub mod search;

pub fn config(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::scope("v2")
			.configure(log::http_v2)
			.configure(package::http_v2)
			.configure(scope::http_v2)
			.configure(identity::http_v2)
			.configure(search::http_v2),
	);
}
