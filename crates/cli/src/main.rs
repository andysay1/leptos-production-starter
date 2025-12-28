use clap::{Parser, Subcommand};
use domain::AuthService;
use shared::config::AppConfig;
use shared::dto::RegisterRequest;
use shared::types::UserRole;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    name = "cmsddrw2-cli",
    version,
    about = "Admin utilities for the Leptos template"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run database migrations
    Migrate,
    /// Seed development data
    Seed {
        #[arg(long, default_value = "admin@example.com")]
        admin_email: String,
        #[arg(long, default_value = "admin1234")]
        admin_password: String,
    },
    /// Create a user with the given credentials
    CreateUser {
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
        #[arg(long, default_value = "user")]
        role: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::from_env()?;
    let db = db::Database::connect(&config.database).await?;
    db.migrate().await?;
    let auth = AuthService::new(Arc::new(db.clone()));

    match cli.command {
        Commands::Migrate => {
            println!("Migrations applied");
        }
        Commands::Seed {
            admin_email,
            admin_password,
        } => {
            seed(&auth, admin_email, admin_password).await?;
        }
        Commands::CreateUser {
            email,
            password,
            role,
        } => {
            let role = parse_role(&role)?;
            let req = RegisterRequest { email, password };
            let user = auth.register(req, Some(role)).await?;
            println!("Created user {} ({:?})", user.email, user.role);
        }
    }

    Ok(())
}

async fn seed(
    auth: &AuthService<db::Database>,
    admin_email: String,
    admin_password: String,
) -> anyhow::Result<()> {
    let admin_req = RegisterRequest {
        email: admin_email.clone(),
        password: admin_password,
    };
    match auth.register(admin_req, Some(UserRole::Admin)).await {
        Ok(user) => println!("Seeded admin user {}", user.email),
        Err(shared::error::AppError::Conflict(_)) => {
            println!("Admin user already exists ({admin_email})")
        }
        Err(err) => return Err(err.into()),
    }

    let user_req = RegisterRequest {
        email: "user@example.com".into(),
        password: "password123".into(),
    };
    match auth.register(user_req, Some(UserRole::User)).await {
        Ok(user) => println!("Seeded demo user {}", user.email),
        Err(shared::error::AppError::Conflict(_)) => println!("Demo user already exists"),
        Err(err) => return Err(err.into()),
    }

    Ok(())
}

fn parse_role(role: &str) -> anyhow::Result<UserRole> {
    match role.to_lowercase().as_str() {
        "admin" => Ok(UserRole::Admin),
        "user" => Ok(UserRole::User),
        other => Err(anyhow::anyhow!("invalid role: {other}")),
    }
}
