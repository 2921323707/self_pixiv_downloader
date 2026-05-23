"use client";

import { useEffect, useState } from "react";
import {
  Check,
  Eye,
  Filter,
  ImageOff,
  Images,
  Loader2,
  RefreshCw,
  Tags,
  Trash2,
  X
} from "lucide-react";
import {
  deleteImages,
  fetchImage,
  fetchImages,
  GalleryImage,
  GalleryImageDetail
} from "../../lib/api";

const filters = [
  { label: "All", value: "include" },
  { label: "Normal", value: "exclude" },
  { label: "R18", value: "only_r18" }
];

export default function GalleryPage() {
  const [images, setImages] = useState<GalleryImage[]>([]);
  const [visibility, setVisibility] = useState("exclude");
  const [tag, setTag] = useState("");
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedImage, setSelectedImage] = useState<GalleryImageDetail | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [selectionMode, setSelectionMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [deleteLoading, setDeleteLoading] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deleteSummary, setDeleteSummary] = useState<string | null>(null);

  async function load(cursor: string | null = null) {
    setLoading(true);
    setError(null);
    try {
      const result = await fetchImages({
        tag: tag.trim() || undefined,
        r18_visibility: visibility,
        limit: 24,
        cursor
      });
      setImages((current) => (cursor ? [...current, ...result.items] : result.items));
      setNextCursor(result.next_cursor);
      if (!cursor) {
        setSelectedIds(new Set());
        setSelectionMode(false);
      }
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Gallery lookup failed");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [visibility]);

  async function openImage(image: GalleryImage) {
    if (selectionMode) {
      toggleSelected(image.image_id);
      return;
    }
    setSelectedImage(null);
    setDetailLoading(true);
    setDetailError(null);
    try {
      setSelectedImage(await fetchImage(image.image_id));
    } catch (caught) {
      setDetailError(caught instanceof Error ? caught.message : "Image detail lookup failed");
    } finally {
      setDetailLoading(false);
    }
  }

  function toggleSelected(imageId: string) {
    setDeleteError(null);
    setDeleteSummary(null);
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(imageId)) {
        next.delete(imageId);
      } else {
        next.add(imageId);
      }
      return next;
    });
  }

  async function deleteSelected() {
    const imageIds = Array.from(selectedIds);
    if (imageIds.length === 0 || deleteLoading) return;
    const confirmed = window.confirm(
      `Delete ${imageIds.length} selected image${imageIds.length === 1 ? "" : "s"} from disk and SQLite?`
    );
    if (!confirmed) return;

    setDeleteLoading(true);
    setDeleteError(null);
    setDeleteSummary(null);
    try {
      const result = await deleteImages(imageIds);
      const deleted = new Set(
        result.items.filter((item) => item.status === "deleted").map((item) => item.image_id)
      );
      setImages((current) => current.filter((image) => !deleted.has(image.image_id)));
      setSelectedIds(new Set());
      if (selectedImage && deleted.has(selectedImage.image_id)) {
        setSelectedImage(null);
      }
      setDeleteSummary(
        result.failed_count > 0
          ? `${result.deleted_count} deleted, ${result.failed_count} failed.`
          : `${result.deleted_count} deleted.`
      );
      if (result.failed_count === 0) {
        setSelectionMode(false);
      }
    } catch (caught) {
      setDeleteError(caught instanceof Error ? caught.message : "Delete failed");
    } finally {
      setDeleteLoading(false);
    }
  }

  return (
    <div className="page-grid">
      <section className="page-heading">
        <div>
          <h1>Gallery</h1>
          <p>{images.length} indexed image{images.length === 1 ? "" : "s"} from SQLite.</p>
        </div>
        <button className="button secondary" onClick={() => load()} type="button">
          {loading ? (
            <Loader2 className="spin" size={16} aria-hidden="true" />
          ) : (
            <RefreshCw size={16} aria-hidden="true" />
          )}
          Refresh
        </button>
      </section>

      <section className="gallery-toolbar">
        <Filter size={17} aria-hidden="true" />
        {filters.map((filter) => (
          <button
            className={`filter-chip ${visibility === filter.value ? "active" : ""}`}
            key={filter.value}
            onClick={() => setVisibility(filter.value)}
            type="button"
          >
            {filter.label}
          </button>
        ))}
        <form
          className="inline-filter"
          onSubmit={(event) => {
            event.preventDefault();
            load();
          }}
        >
          <input
            placeholder="tag"
            value={tag}
            onChange={(event) => setTag(event.target.value)}
          />
          <button className="button secondary" type="submit">
            Apply
          </button>
        </form>
        <button
          className="button secondary"
          onClick={() => {
            setSelectionMode((current) => !current);
            setDeleteError(null);
            setDeleteSummary(null);
            if (selectionMode) {
              setSelectedIds(new Set());
            }
          }}
          type="button"
        >
          <Check size={16} aria-hidden="true" />
          {selectionMode ? "Cancel" : "Select"}
        </button>
        <button
          className="button danger"
          disabled={!selectionMode || selectedIds.size === 0 || deleteLoading}
          onClick={deleteSelected}
          type="button"
        >
          {deleteLoading ? (
            <Loader2 className="spin" size={16} aria-hidden="true" />
          ) : (
            <Trash2 size={16} aria-hidden="true" />
          )}
          Delete {selectedIds.size > 0 ? selectedIds.size : ""}
        </button>
      </section>

      {error ? <div className="error-box">{error}</div> : null}
      {deleteError ? <div className="error-box">{deleteError}</div> : null}
      {deleteSummary ? <div className="success-box">{deleteSummary}</div> : null}

      {images.length > 0 ? (
        <section className="gallery-grid" aria-label="Indexed images">
          {images.map((image) => {
            const selected = selectedIds.has(image.image_id);
            return (
              <button
                aria-pressed={selectionMode ? selected : undefined}
                className={`image-card ${selected ? "selected" : ""}`}
                key={image.image_id}
                onClick={() => openImage(image)}
                type="button"
              >
                <div
                  className={`image-placeholder category-${image.category}`}
                  style={{
                    aspectRatio:
                      image.width && image.height ? `${image.width} / ${image.height}` : "4 / 5"
                  }}
                >
                  {image.thumbnail_url || image.preview_url ? (
                    <img
                      alt={image.title || `Pixiv ${image.pixiv_id} page ${image.page_index}`}
                      src={image.thumbnail_url || image.preview_url || ""}
                    />
                  ) : (
                    <ImageOff size={22} aria-hidden="true" />
                  )}
                  {selectionMode ? (
                    <span className={`selection-badge ${selected ? "active" : ""}`}>
                      <Check size={13} aria-hidden="true" />
                    </span>
                  ) : null}
                  <span>#{image.pixiv_id}_p{image.page_index}</span>
                </div>
                <div className="image-card-title">{image.title || "Untitled"}</div>
                <div className="image-card-meta">
                  <span>
                    <Images size={14} aria-hidden="true" />
                    {image.category}
                  </span>
                  <span>
                    <Tags size={14} aria-hidden="true" />
                    {image.tags.slice(0, 2).join(", ") || "no tags"}
                  </span>
                </div>
              </button>
            );
          })}
        </section>
      ) : (
        <section className="task-detail empty-state">
          <Images size={22} aria-hidden="true" />
          <p>No indexed images match the current filters.</p>
        </section>
      )}

      {nextCursor ? (
        <button className="button secondary load-more" onClick={() => load(nextCursor)} type="button">
          Load more
        </button>
      ) : null}

      {detailLoading || detailError || selectedImage ? (
        <div className="drawer-backdrop" role="presentation">
          <aside className="image-detail-drawer" aria-label="Image detail drawer">
            <div className="image-detail-head">
              <div>
                <h2>{selectedImage?.title || "Image preview"}</h2>
                <p>
                  {selectedImage
                    ? `#${selectedImage.pixiv_id}_p${selectedImage.page_index}`
                    : "Loading image detail"}
                </p>
              </div>
              <button
                className="icon-button"
                onClick={() => {
                  setSelectedImage(null);
                  setDetailError(null);
                }}
                type="button"
                aria-label="Close image detail"
              >
                <X size={16} aria-hidden="true" />
              </button>
            </div>

            {detailError ? <div className="error-box">{detailError}</div> : null}
            {detailLoading ? (
              <div className="task-detail empty-state">
                <Loader2 className="spin" size={20} aria-hidden="true" />
                <p>Loading preview.</p>
              </div>
            ) : null}
            {selectedImage ? (
              <div className="image-detail-body">
                <div className="image-preview-frame">
                  {selectedImage.preview_url ? (
                    <img
                      alt={selectedImage.title || `Pixiv ${selectedImage.pixiv_id}`}
                      src={selectedImage.preview_url}
                    />
                  ) : (
                    <ImageOff size={28} aria-hidden="true" />
                  )}
                </div>
                <div className="image-detail-facts">
                  <span>
                    <Images size={14} aria-hidden="true" />
                    {selectedImage.category}
                  </span>
                  <span>
                    <Eye size={14} aria-hidden="true" />
                    {selectedImage.width && selectedImage.height
                      ? `${selectedImage.width} x ${selectedImage.height}`
                      : "unknown size"}
                  </span>
                  <span>
                    <Tags size={14} aria-hidden="true" />
                    {selectedImage.tags.join(", ") || "no tags"}
                  </span>
                  {selectedImage.sources.map((source) => (
                    <span key={`${source.source}-${source.task_id || source.created_at}`}>
                      {source.source}
                    </span>
                  ))}
                </div>
              </div>
            ) : null}
          </aside>
        </div>
      ) : null}
    </div>
  );
}
