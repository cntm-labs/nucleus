use std::sync::Arc;

use nucleus_core::error::{AppError, OrgError};
use nucleus_core::pagination::{PaginatedResponse, PaginationParams};
use nucleus_core::types::{OrgId, ProjectId, RoleId, UserId};
use nucleus_core::validation;
use nucleus_db::repos::org_repo::{
    NewOrgMember, NewOrganization, OrgMember, OrgRepository, Organization,
};

pub struct OrgService {
    org_repo: Arc<dyn OrgRepository>,
}

impl OrgService {
    pub fn new(org_repo: Arc<dyn OrgRepository>) -> Self {
        Self { org_repo }
    }

    /// Create a new organization.
    pub async fn create_org(
        &self,
        project_id: &ProjectId,
        name: &str,
        slug: &str,
        created_by: &UserId,
    ) -> Result<Organization, AppError> {
        validation::validate_slug(slug)?;

        if self
            .org_repo
            .find_by_slug(project_id, slug)
            .await?
            .is_some()
        {
            return Err(AppError::Org(OrgError::SlugTaken));
        }

        let new_org = NewOrganization {
            project_id: *project_id,
            name: name.to_string(),
            slug: slug.to_string(),
            logo_url: None,
            metadata: None,
            max_members: None,
            created_by: Some(*created_by),
        };

        self.org_repo.create(&new_org).await
    }

    /// Get an organization by ID.
    pub async fn get_org(
        &self,
        project_id: &ProjectId,
        org_id: &OrgId,
    ) -> Result<Organization, AppError> {
        self.org_repo
            .find_by_id(project_id, org_id)
            .await?
            .ok_or(AppError::Org(OrgError::NotFound))
    }

    /// Get an organization by slug.
    pub async fn get_org_by_slug(
        &self,
        project_id: &ProjectId,
        slug: &str,
    ) -> Result<Organization, AppError> {
        self.org_repo
            .find_by_slug(project_id, slug)
            .await?
            .ok_or(AppError::Org(OrgError::NotFound))
    }

    /// List organizations for a project.
    pub async fn list_orgs(
        &self,
        project_id: &ProjectId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<Organization>, AppError> {
        self.org_repo.list_by_project(project_id, params).await
    }

    /// List organizations a specific user belongs to.
    pub async fn list_user_orgs(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<Vec<Organization>, AppError> {
        self.org_repo.list_by_user(project_id, user_id).await
    }

    /// Add a member to an organization.
    pub async fn add_member(
        &self,
        org_id: &OrgId,
        user_id: &UserId,
        role_id: &RoleId,
        invited_by: Option<&UserId>,
    ) -> Result<OrgMember, AppError> {
        let member = NewOrgMember {
            org_id: *org_id,
            user_id: *user_id,
            role_id: *role_id,
            invited_by: invited_by.copied(),
        };
        self.org_repo.add_member(&member).await
    }

    /// Remove a member from an organization.
    pub async fn remove_member(&self, org_id: &OrgId, user_id: &UserId) -> Result<(), AppError> {
        self.org_repo.remove_member(org_id, user_id).await
    }

    /// Update a member's role in an organization.
    pub async fn update_member_role(
        &self,
        org_id: &OrgId,
        user_id: &UserId,
        role_id: &RoleId,
    ) -> Result<(), AppError> {
        self.org_repo
            .update_member_role(org_id, user_id, role_id)
            .await
    }

    /// List members of an organization.
    pub async fn list_members(
        &self,
        org_id: &OrgId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<OrgMember>, AppError> {
        self.org_repo.list_members(org_id, params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Mutex;
    use uuid::Uuid;

    /// In-memory mock of OrgRepository for testing.
    struct MockOrgRepo {
        orgs: Mutex<Vec<Organization>>,
        members: Mutex<Vec<OrgMember>>,
    }

    impl MockOrgRepo {
        fn new() -> Self {
            Self {
                orgs: Mutex::new(Vec::new()),
                members: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl OrgRepository for MockOrgRepo {
        async fn create(&self, org: &NewOrganization) -> Result<Organization, AppError> {
            let now = Utc::now();
            let created = Organization {
                id: OrgId::new(),
                project_id: org.project_id,
                name: org.name.clone(),
                slug: org.slug.clone(),
                logo_url: org.logo_url.clone(),
                metadata: org.metadata.clone().unwrap_or(serde_json::json!({})),
                max_members: org.max_members,
                created_by: org.created_by,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            };
            self.orgs.lock().unwrap().push(created.clone());
            Ok(created)
        }

        async fn find_by_id(
            &self,
            project_id: &ProjectId,
            org_id: &OrgId,
        ) -> Result<Option<Organization>, AppError> {
            let orgs = self.orgs.lock().unwrap();
            Ok(orgs
                .iter()
                .find(|o| o.project_id == *project_id && o.id == *org_id && o.deleted_at.is_none())
                .cloned())
        }

        async fn find_by_slug(
            &self,
            project_id: &ProjectId,
            slug: &str,
        ) -> Result<Option<Organization>, AppError> {
            let orgs = self.orgs.lock().unwrap();
            Ok(orgs
                .iter()
                .find(|o| o.project_id == *project_id && o.slug == slug && o.deleted_at.is_none())
                .cloned())
        }

        async fn list_by_project(
            &self,
            project_id: &ProjectId,
            params: &PaginationParams,
        ) -> Result<PaginatedResponse<Organization>, AppError> {
            let orgs = self.orgs.lock().unwrap();
            let filtered: Vec<_> = orgs
                .iter()
                .filter(|o| o.project_id == *project_id && o.deleted_at.is_none())
                .cloned()
                .collect();
            let limit = params.effective_limit() as usize;
            let data: Vec<_> = filtered.into_iter().take(limit).collect();
            Ok(PaginatedResponse {
                data,
                has_more: false,
                next_cursor: None,
                total_count: None,
            })
        }

        async fn list_by_user(
            &self,
            project_id: &ProjectId,
            user_id: &UserId,
        ) -> Result<Vec<Organization>, AppError> {
            let orgs = self.orgs.lock().unwrap();
            let members = self.members.lock().unwrap();
            let member_org_ids: Vec<OrgId> = members
                .iter()
                .filter(|m| m.user_id == *user_id)
                .map(|m| m.org_id)
                .collect();
            Ok(orgs
                .iter()
                .filter(|o| {
                    o.project_id == *project_id
                        && member_org_ids.contains(&o.id)
                        && o.deleted_at.is_none()
                })
                .cloned()
                .collect())
        }

        async fn add_member(&self, member: &NewOrgMember) -> Result<OrgMember, AppError> {
            let now = Utc::now();
            let created = OrgMember {
                id: Uuid::new_v4(),
                org_id: member.org_id,
                user_id: member.user_id,
                role_id: member.role_id,
                invited_by: member.invited_by,
                joined_at: Some(now),
                created_at: now,
            };
            self.members.lock().unwrap().push(created.clone());
            Ok(created)
        }

        async fn remove_member(&self, org_id: &OrgId, user_id: &UserId) -> Result<(), AppError> {
            let mut members = self.members.lock().unwrap();
            members.retain(|m| !(m.org_id == *org_id && m.user_id == *user_id));
            Ok(())
        }

        async fn update_member_role(
            &self,
            org_id: &OrgId,
            user_id: &UserId,
            role_id: &RoleId,
        ) -> Result<(), AppError> {
            let mut members = self.members.lock().unwrap();
            for m in members.iter_mut() {
                if m.org_id == *org_id && m.user_id == *user_id {
                    m.role_id = *role_id;
                }
            }
            Ok(())
        }

        async fn list_members(
            &self,
            org_id: &OrgId,
            params: &PaginationParams,
        ) -> Result<PaginatedResponse<OrgMember>, AppError> {
            let members = self.members.lock().unwrap();
            let filtered: Vec<_> = members
                .iter()
                .filter(|m| m.org_id == *org_id)
                .cloned()
                .collect();
            let limit = params.effective_limit() as usize;
            let data: Vec<_> = filtered.into_iter().take(limit).collect();
            Ok(PaginatedResponse {
                data,
                has_more: false,
                next_cursor: None,
                total_count: None,
            })
        }
    }

    fn make_service() -> OrgService {
        OrgService::new(Arc::new(MockOrgRepo::new()))
    }

    fn default_params() -> PaginationParams {
        PaginationParams {
            limit: 20,
            cursor: None,
        }
    }

    #[tokio::test]
    async fn create_org_success() {
        let svc = make_service();
        let pid = ProjectId::new();
        let uid = UserId::new();

        let org = svc
            .create_org(&pid, "Acme Corp", "acme-corp", &uid)
            .await
            .unwrap();
        assert_eq!(org.name, "Acme Corp");
        assert_eq!(org.slug, "acme-corp");
        assert_eq!(org.project_id, pid);
        assert_eq!(org.created_by, Some(uid));
    }

    #[tokio::test]
    async fn create_org_rejects_duplicate_slug() {
        let svc = make_service();
        let pid = ProjectId::new();
        let uid = UserId::new();

        svc.create_org(&pid, "Acme Corp", "acme-corp", &uid)
            .await
            .unwrap();
        let result = svc.create_org(&pid, "Other Corp", "acme-corp", &uid).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Org(OrgError::SlugTaken)));
    }

    #[tokio::test]
    async fn create_org_rejects_invalid_slug() {
        let svc = make_service();
        let pid = ProjectId::new();
        let uid = UserId::new();

        // Uppercase is invalid
        let result = svc.create_org(&pid, "Test", "Invalid-Slug", &uid).await;
        assert!(result.is_err());

        // Too short
        let result = svc.create_org(&pid, "Test", "ab", &uid).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_org_returns_not_found() {
        let svc = make_service();
        let pid = ProjectId::new();
        let oid = OrgId::new();

        let result = svc.get_org(&pid, &oid).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::Org(OrgError::NotFound)
        ));
    }

    #[tokio::test]
    async fn add_and_list_members() {
        let svc = make_service();
        let pid = ProjectId::new();
        let uid = UserId::new();

        let org = svc
            .create_org(&pid, "Acme", "acme-org", &uid)
            .await
            .unwrap();

        let user1 = UserId::new();
        let user2 = UserId::new();
        let role = RoleId::new();

        svc.add_member(&org.id, &user1, &role, Some(&uid))
            .await
            .unwrap();
        svc.add_member(&org.id, &user2, &role, Some(&uid))
            .await
            .unwrap();

        let members = svc.list_members(&org.id, &default_params()).await.unwrap();
        assert_eq!(members.data.len(), 2);
    }

    #[tokio::test]
    async fn remove_member() {
        let svc = make_service();
        let pid = ProjectId::new();
        let uid = UserId::new();

        let org = svc
            .create_org(&pid, "Acme", "acme-rem", &uid)
            .await
            .unwrap();

        let member_uid = UserId::new();
        let role = RoleId::new();
        svc.add_member(&org.id, &member_uid, &role, None)
            .await
            .unwrap();

        let members = svc.list_members(&org.id, &default_params()).await.unwrap();
        assert_eq!(members.data.len(), 1);

        svc.remove_member(&org.id, &member_uid).await.unwrap();

        let members = svc.list_members(&org.id, &default_params()).await.unwrap();
        assert_eq!(members.data.len(), 0);
    }

    #[tokio::test]
    async fn cross_tenant_org_isolation() {
        let svc = make_service();
        let project_a = ProjectId::new();
        let project_b = ProjectId::new();
        let uid = UserId::new();

        let org_a = svc
            .create_org(&project_a, "Org A", "org-aaa", &uid)
            .await
            .unwrap();

        // Org in project A should not be visible from project B
        let result = svc.get_org(&project_b, &org_a.id).await;
        assert!(matches!(
            result.unwrap_err(),
            AppError::Org(OrgError::NotFound)
        ));

        let result = svc.get_org_by_slug(&project_b, "org-aaa").await;
        assert!(matches!(
            result.unwrap_err(),
            AppError::Org(OrgError::NotFound)
        ));

        // Same slug can be used in a different project
        let org_b = svc.create_org(&project_b, "Org B", "org-aaa", &uid).await;
        assert!(org_b.is_ok());
    }
}
