use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Pagination request parameters
#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Number of items per page (max 100)
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
        }
    }
}

impl PaginationParams {
    pub fn new(page: u32, page_size: u32) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.min(100).max(1),
        }
    }

    /// Calculate the offset for database queries
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }

    /// Get the limit for database queries
    pub fn limit(&self) -> u32 {
        self.page_size
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// The data items for the current page
    pub data: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginationMeta {
    /// Current page number (1-indexed)
    pub page: u32,

    /// Number of items per page
    pub page_size: u32,

    /// Total number of items
    pub total_items: u64,

    /// Total number of pages
    pub total_pages: u32,
}

impl PaginationMeta {
    pub fn new(page: u32, page_size: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;
        Self {
            page,
            page_size,
            total_items,
            total_pages,
        }
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, params: &PaginationParams, total_items: u64) -> Self {
        Self {
            data,
            pagination: PaginationMeta::new(params.page, params.page_size, total_items),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_offset() {
        let params = PaginationParams::new(1, 20);
        assert_eq!(params.offset(), 0);

        let params = PaginationParams::new(2, 20);
        assert_eq!(params.offset(), 20);

        let params = PaginationParams::new(3, 10);
        assert_eq!(params.offset(), 20);
    }

    #[test]
    fn test_pagination_params_validation() {
        let params = PaginationParams::new(0, 20);
        assert_eq!(params.page, 1); // Minimum is 1

        let params = PaginationParams::new(1, 0);
        assert_eq!(params.page_size, 1); // Minimum is 1

        let params = PaginationParams::new(1, 200);
        assert_eq!(params.page_size, 100); // Maximum is 100
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(1, 20, 100);
        assert_eq!(meta.total_pages, 5);

        let meta = PaginationMeta::new(1, 20, 95);
        assert_eq!(meta.total_pages, 5);

        let meta = PaginationMeta::new(1, 20, 101);
        assert_eq!(meta.total_pages, 6);
    }

    #[test]
    fn test_paginated_response() {
        let data = vec![1, 2, 3, 4, 5];
        let params = PaginationParams::new(1, 5);
        let response = PaginatedResponse::new(data, &params, 25);

        assert_eq!(response.data.len(), 5);
        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.page_size, 5);
        assert_eq!(response.pagination.total_items, 25);
        assert_eq!(response.pagination.total_pages, 5);
    }
}
