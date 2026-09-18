//! 管理端权限校验与请求审计上下文。

use axum::{
    Router,
    extract::{FromRequestParts, State},
    http::{HeaderMap, Method, StatusCode, request::Parts},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use gateway_admin::model::{
    auth::{AdminPrincipal, AdminRequestContext, AdminRole, AdminUser, CreateAdminUser},
};
use serde::{Deserialize, Serialize};
use tower_http::request_id::RequestId;

use crate::{auth::SessionState, session_cookie};

use super::{
    AdminAuth, AdminEnvelope, AdminError, AdminJson, AdminResponse, wire::map_admin_service_error,
};

const REQUEST_ID_HEADER: &str = "x-request-id";

/// 已通过管理员会话或部署级管理 API Key 鉴权的请求。
pub struct AdminAuth {
    context: AdminRequestContext,
}

impl AdminAuth {
    #[must_use]
    pub const fn context(&self) -> &AdminRequestContext {
        &self.context
    }
}

impl<S> FromRequestParts<S> for AdminAuth
where
    S: SessionState + Send + Sync,
{
    type Rejection = AdminError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let principal = require_admin_auth(state, &parts.headers).await?;
        if let AdminPrincipal::Session { admin_user_id } = &principal {
            let role = state
                .admin_services()
                .auth()
                .admin_role(admin_user_id)
                .await
                .map_err(map_admin_service_error)?;
            if role.is_read_only() && !matches!(parts.method, Method::GET | Method::HEAD) {
                return Err(AdminError::forbidden());
            }
        }
        let request_id = admin_request_id(parts).ok_or_else(AdminError::internal)?;
        Ok(Self {
            context: AdminRequestContext {
                principal,
                request_id,
            },
        })
    }
}

/// request-id 层按配置的 header 名注入，同时写入与名字无关的扩展；
/// 优先读扩展，使自定义 header 名不会让管理请求失去请求上下文。
/// header 回退覆盖未装配该层的调用方。
fn admin_request_id(parts: &Parts) -> Option<String> {
    parts
        .extensions
        .get::<RequestId>()
        .map(RequestId::header_value)
        .or_else(|| parts.headers.get(REQUEST_ID_HEADER))
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub async fn require_admin_session<S>(state: &S, headers: &HeaderMap) -> Result<String, AdminError>
where
    S: SessionState + Send + Sync,
{
    match state
        .admin_services()
        .auth()
        .resolve_admin_user_id(session_cookie::value(headers).as_deref())
        .await
    {
        Ok(Some(admin_user_id)) => Ok(admin_user_id),
        Ok(None) => Err(AdminError::session_required()),
        Err(error) => Err(map_admin_service_error(error)),
    }
}

async fn require_admin_auth<S>(state: &S, headers: &HeaderMap) -> Result<AdminPrincipal, AdminError>
where
    S: SessionState + Send + Sync,
{
    if let Some(api_key) = admin_api_key_header(headers) {
        return match state
            .admin_services()
            .auth()
            .verify_admin_api_key(&api_key)
            .await
        {
            Ok(true) => Ok(AdminPrincipal::ApiKey),
            Ok(false) => Err(AdminError::invalid_admin_api_key()),
            Err(error) => Err(map_admin_service_error(error)),
        };
    }

    require_admin_session(state, headers)
        .await
        .map(|admin_user_id| AdminPrincipal::Session { admin_user_id })
}

fn admin_api_key_header(headers: &HeaderMap) -> Option<String> {
    let value = headers.get("x-api-key")?.to_str().ok()?.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// 管理员账户创建请求。
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateAdminUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
}

/// 管理员账户列表只返回公开的身份和权限信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserView {
    pub username: String,
    pub role: &'static str,
    pub created_at: DateTime<Utc>,
}

impl From<AdminUser> for AdminUserView {
    fn from(user: AdminUser) -> Self {
        Self {
            username: user.username,
            role: user.role.as_str(),
            created_at: user.created_at,
        }
    }
}

/// 构造管理员账户管理路由。
pub(crate) fn router<S>() -> Router<S>
where
    S: SessionState + Clone + Send + Sync + 'static,
{
    Router::new().route(
        "/api/admin/auth/users",
        get(list_admin_users::<S>).post(create_admin_user::<S>),
    )
}

async fn list_admin_users<S>(
    _auth: AdminAuth,
    State(state): State<S>,
) -> Result<impl IntoResponse, AdminError>
where
    S: SessionState + Send + Sync,
{
    let users = state
        .admin_services()
        .auth()
        .list_admin_users()
        .await
        .map_err(map_admin_service_error)?;
    Ok(AdminResponse::new(
        StatusCode::OK,
        AdminEnvelope::ok(
            users
                .into_iter()
                .map(AdminUserView::from)
                .collect::<Vec<_>>(),
        ),
    ))
}

async fn create_admin_user<S>(
    _auth: AdminAuth,
    State(state): State<S>,
    AdminJson(request): AdminJson<CreateAdminUserRequest>,
) -> Result<impl IntoResponse, AdminError>
where
    S: SessionState + Send + Sync,
{
    let role = AdminRole::parse(request.role.trim())
        .ok_or_else(|| AdminError::bad_request("管理员角色不合法"))?;
    let user = state
        .admin_services()
        .auth()
        .create_admin_user(CreateAdminUser {
            username: request.username,
            password: request.password,
            role,
        })
        .await
        .map_err(map_admin_service_error)?;
    Ok(AdminResponse::new(
        StatusCode::CREATED,
        AdminEnvelope::ok(AdminUserView::from(user)),
    ))
}
