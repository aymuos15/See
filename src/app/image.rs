//! Image loading and caching handlers

use crate::constants::FULL_IMAGE_DELAY_MS;
use std::path::Path;
use std::time::Instant;

use super::App;

impl App {
    /// Handle image loaded from background worker (full quality)
    pub fn handle_image_loaded(
        &mut self,
        path: &Path,
        result: &anyhow::Result<image::DynamicImage>,
    ) {
        if let (Ok(dyn_img), Some(ref picker)) = (result, &self.image_picker) {
            let protocol = picker.new_resize_protocol(dyn_img.clone());
            // Store protocol by canonical path for consistent lookup
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            self.image_protocols
                .insert(canonical_path.clone(), protocol);
            // Mark as full quality
            self.full_quality_images.insert(canonical_path);
        }
    }

    /// Handle thumbnail loaded from background worker
    pub fn handle_thumbnail_loaded(
        &mut self,
        path: &Path,
        result: &anyhow::Result<image::DynamicImage>,
    ) {
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Only use thumbnail if we don't already have full quality
        if self.full_quality_images.contains(&canonical_path) {
            return;
        }

        if let (Ok(dyn_img), Some(ref picker)) = (result, &self.image_picker) {
            let protocol = picker.new_resize_protocol(dyn_img.clone());
            self.image_protocols
                .insert(canonical_path.clone(), protocol);
            // Schedule full quality load
            self.schedule_full_quality_load(&canonical_path);
        }
    }

    /// Schedule a full quality image load after the debounce delay
    pub fn schedule_full_quality_load(&mut self, path: &Path) {
        let deadline = Instant::now() + std::time::Duration::from_millis(FULL_IMAGE_DELAY_MS);
        self.pending_full_quality = Some((path.to_path_buf(), deadline));
    }

    /// Check if pending full quality load should be triggered
    /// Returns true if a full quality load was requested
    pub fn check_pending_full_quality(&mut self) -> bool {
        if let Some((ref path, deadline)) = self.pending_full_quality {
            if Instant::now() >= deadline {
                // Don't reload if already full quality
                if !self.full_quality_images.contains(path) {
                    self.worker.request_image_load(path);
                }
                self.pending_full_quality = None;
                return true;
            }
        }
        false
    }

    /// Cancel pending full quality load (called on navigation)
    pub fn cancel_pending_full_quality(&mut self) {
        self.pending_full_quality = None;
    }
}
