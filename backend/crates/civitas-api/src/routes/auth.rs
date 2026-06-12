//! Authentication routes.
//!
//! Registration and password reset issue tokens and hand them to the
//! mailer (`crate::mailer`); delivery runs in the background so handlers
//! never block on SMTP. Endpoints that take an email address respond
//! identically whether or not an account exists — no enumeration.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::cookie::CookieJar;

use civitas_auth::session::DEFAULT_LIFETIME;
use civitas_auth::verification::VerificationProvider;
use civitas_auth::{login, password_reset, register, session};

use crate::auth_extractor::AuthSession;
use crate::cookies::{clear_session_cookie, session_cookie};
use crate::dto::{
    LoginRequest, PasswordResetCompleteRequest, PasswordResetRequest, RegisterRequest,
    RegisterResponse, ResendVerificationRequest, UserResponse, VerifyEmailRequest,
};
use crate::error::{ApiError, ApiResult};
use crate::mailer;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/me", get(me_handler))
        .route("/verify-email", post(verify_email_handler))
        .route("/resend-verification", post(resend_verification_handler))
        .route(
            "/password-reset/request",
            post(password_reset_request_handler),
        )
        .route(
            "/password-reset/complete",
            post(password_reset_complete_handler),
        )
}

async fn register_handler(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<RegisterResponse>)> {
    let registered = register::register(
        state.pool(),
        register::NewRegistration {
            email: &body.email,
            password: &body.password,
            display_name: &body.display_name,
        },
        state.email_verification(),
    )
    .await
    .map_err(ApiError::from)?;

    let user = civitas_db::users::find_by_id(state.pool(), registered.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    mailer::send_in_background(
        state.mailer(),
        mailer::verification_mail(
            &state.config().public_base_url,
            &user.email,
            &registered.verification.plaintext,
        ),
    );

    let user: UserResponse = user.into();

    let dev_verification_token = if state.config().dev_return_verification_token {
        Some(registered.verification.plaintext)
    } else {
        None
    };

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            email_verified: user.email_verified,
            phone_verified: user.phone_verified,
            created_at: user.created_at,
            dev_verification_token,
        }),
    ))
}

async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> ApiResult<(CookieJar, Json<UserResponse>)> {
    let issued = login::authenticate(
        state.pool(),
        &body.email,
        &body.password,
        None, // user_agent: TODO surface from request headers
        None, // ip_address: TODO surface from connect-info
        DEFAULT_LIFETIME,
    )
    .await
    .map_err(ApiError::from)?;

    let user = civitas_db::users::find_by_id(state.pool(), issued.row.user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::Internal(anyhow::anyhow!(
            "user disappeared after login"
        )))?;

    let cookie = session_cookie(
        &state.config().cookie,
        issued.cookie_value,
        DEFAULT_LIFETIME.num_days(),
    );
    Ok((jar.add(cookie), Json(user.into())))
}

async fn logout_handler(
    State(state): State<AppState>,
    auth: AuthSession,
    jar: CookieJar,
) -> ApiResult<(CookieJar, StatusCode)> {
    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    session::revoke(&mut tx, auth.user.id, auth.session.id)
        .await
        .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;

    let cleared = clear_session_cookie(&state.config().cookie);
    Ok((jar.add(cleared), StatusCode::NO_CONTENT))
}

async fn me_handler(auth: AuthSession) -> Json<UserResponse> {
    Json(auth.user.into())
}

async fn verify_email_handler(
    State(state): State<AppState>,
    Json(body): Json<VerifyEmailRequest>,
) -> ApiResult<Json<UserResponse>> {
    let result = state
        .email_verification()
        .complete(state.pool(), &body.token)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(result.user.into()))
}

async fn resend_verification_handler(
    State(state): State<AppState>,
    Json(body): Json<ResendVerificationRequest>,
) -> ApiResult<StatusCode> {
    // We deliberately do not reveal whether the email matched a user.
    let user = civitas_db::users::find_by_email(state.pool(), &body.email)
        .await
        .map_err(ApiError::from)?;

    if let Some(user) = user {
        if user.is_active() && !user.is_email_verified() {
            let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
            let issued = state
                .email_verification()
                .initiate(&mut tx, user.id, &user.email)
                .await
                .map_err(ApiError::from)?;
            tx.commit().await.map_err(ApiError::from)?;

            mailer::send_in_background(
                state.mailer(),
                mailer::verification_mail(
                    &state.config().public_base_url,
                    &user.email,
                    &issued.plaintext,
                ),
            );
        }
    }
    Ok(StatusCode::ACCEPTED)
}

async fn password_reset_request_handler(
    State(state): State<AppState>,
    Json(body): Json<PasswordResetRequest>,
) -> ApiResult<StatusCode> {
    // We deliberately do not reveal whether the email matched a user.
    if let Some(issued) = password_reset::request(
        state.pool(),
        &body.email,
        password_reset::DEFAULT_RESET_LIFETIME,
    )
    .await
    .map_err(ApiError::from)?
    {
        mailer::send_in_background(
            state.mailer(),
            mailer::password_reset_mail(
                &state.config().public_base_url,
                &body.email,
                &issued.plaintext,
            ),
        );
    }
    Ok(StatusCode::ACCEPTED)
}

async fn password_reset_complete_handler(
    State(state): State<AppState>,
    Json(body): Json<PasswordResetCompleteRequest>,
) -> ApiResult<StatusCode> {
    password_reset::complete(state.pool(), &body.token, &body.new_password)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
