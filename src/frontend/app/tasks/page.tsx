"use client";

import { FormEvent, useCallback, useEffect, useState } from "react";
import { ListChecks, Loader2, RefreshCw, Search, X } from "lucide-react";
import { fetchTask, fetchTasks, TaskSnapshot, TaskSummary } from "../../lib/api";
import { StatusBadge } from "../../components/StatusBadge";

export default function TasksPage() {
  const [taskId, setTaskId] = useState("");
  const [trackedId, setTrackedId] = useState("");
  const [task, setTask] = useState<TaskSnapshot | null>(null);
  const [tasks, setTasks] = useState<TaskSummary[]>([]);
  const [recentLimit, setRecentLimit] = useState(10);
  const [hasMoreTasks, setHasMoreTasks] = useState(false);
  const [detailOpen, setDetailOpen] = useState(false);
  const [detailLoading, setDetailLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const queryTask = params.get("task");
    if (queryTask) {
      setTaskId(queryTask);
      setTrackedId(queryTask);
      setDetailOpen(true);
    }
  }, []);

  async function loadTaskList() {
    try {
      const result = await fetchTasks({ limit: recentLimit });
      setTasks(result.items);
      setHasMoreTasks(Boolean(result.next_cursor));
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Task list lookup failed");
    }
  }

  useEffect(() => {
    loadTaskList();
    const interval = window.setInterval(loadTaskList, 3000);

    return () => {
      window.clearInterval(interval);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [recentLimit]);

  useEffect(() => {
    if (!trackedId || !detailOpen) {
      return;
    }

    let active = true;

    async function load() {
      setDetailLoading(true);
      try {
        const snapshot = await fetchTask(trackedId);
        if (active) {
          setTask(snapshot);
          setError(null);
          loadTaskList();
        }
      } catch (caught) {
        if (active) {
          setError(caught instanceof Error ? caught.message : "Task lookup failed");
        }
      } finally {
        if (active) {
          setDetailLoading(false);
        }
      }
    }

    load();
    const interval = window.setInterval(() => {
      if (!task?.status || !["completed", "failed", "cancelled"].includes(task.status)) {
        load();
      }
    }, 1600);

    return () => {
      active = false;
      window.clearInterval(interval);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [trackedId, detailOpen, task?.status]);

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const nextTaskId = taskId.trim();
    if (!nextTaskId) return;
    setTask(null);
    setTrackedId(nextTaskId);
    setDetailOpen(true);
  }

  function openTask(nextTaskId: string) {
    setTaskId(nextTaskId);
    setTask(null);
    setTrackedId(nextTaskId);
    setDetailOpen(true);
  }

  const closeDetail = useCallback(() => {
    setDetailOpen(false);
  }, []);

  useEffect(() => {
    function handleKeyDown(event: globalThis.KeyboardEvent) {
      if (event.key === "Escape") {
        closeDetail();
      }
    }

    if (detailOpen) {
      window.addEventListener("keydown", handleKeyDown);
    }

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [closeDetail, detailOpen]);

  const total = task?.progress_total || 1;
  const progress = task ? ((task.progress_done + task.progress_failed) / total) * 100 : 0;

  return (
    <div className="page-grid">
      <section className="page-heading">
        <div>
          <h1>Task Panel</h1>
          <p>Recent tasks stay compact; details open in a live progress modal.</p>
        </div>
        <StatusBadge status={task?.status || "pending"} />
      </section>

      <form className="task-search" onSubmit={submit}>
        <Search size={18} aria-hidden="true" />
        <input
          placeholder="task-..."
          value={taskId}
          onChange={(event) => setTaskId(event.target.value)}
        />
        <button className="button secondary" type="submit">
          <RefreshCw size={16} aria-hidden="true" />
          Track
        </button>
      </form>

      {error ? <div className="error-box">{error}</div> : null}

      <section className="task-detail">
        <div className="panel-title">
          <ListChecks size={18} aria-hidden="true" />
          <h2>Recent Tasks</h2>
        </div>
        {tasks.length > 0 ? (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Task</th>
                  <th>Type</th>
                  <th>Status</th>
                  <th>Progress</th>
                </tr>
              </thead>
              <tbody>
                {tasks.map((item) => (
                  <tr
                    className="click-row"
                    key={item.task_id}
                    onClick={() => openTask(item.task_id)}
                  >
                    <td>{item.task_id}</td>
                    <td>{item.type}</td>
                    <td>
                      <StatusBadge status={item.status} />
                    </td>
                    <td>
                      {item.progress_done + item.progress_failed}/{item.progress_total || 1}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="quiet">No tasks have been persisted yet.</p>
        )}
        {hasMoreTasks ? (
          <button
            className="button secondary load-more"
            onClick={() => setRecentLimit((current) => current + 10)}
            type="button"
          >
            Show more
          </button>
        ) : null}
      </section>

      {detailOpen ? (
        <div className="modal-backdrop" onClick={closeDetail} role="presentation">
          <section
            className="task-modal"
            aria-label="Task detail modal"
            role="dialog"
            aria-modal="true"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="image-detail-head">
              <div className="panel-title modal-title">
                <ListChecks size={18} aria-hidden="true" />
                <h2>{task ? task.task_id : trackedId || "Task detail"}</h2>
              </div>
              <button
                className="icon-button"
                onClick={closeDetail}
                type="button"
                aria-label="Close task detail"
              >
                <X size={16} aria-hidden="true" />
              </button>
            </div>

            {detailLoading && !task ? (
              <div className="task-detail empty-state">
                <Loader2 className="spin" size={20} aria-hidden="true" />
                <p>Loading task detail.</p>
              </div>
            ) : null}

            {task ? (
              <>
                <div className="task-meta">
                  <span>Type: {task.type}</span>
                  <span>Current: {task.current_item || "none"}</span>
                  <span>Done: {task.progress_done}</span>
                  <span>Failed: {task.progress_failed}</span>
                </div>
                <div className="progress-track" aria-label="Task progress">
                  <span style={{ width: `${Math.min(progress, 100)}%` }} />
                </div>
                {task.error_message ? (
                  <div className="error-box">
                    {task.error_code}: {task.error_message}
                  </div>
                ) : null}
                <div className="table-wrap">
                  <table>
                    <thead>
                      <tr>
                        <th>Item</th>
                        <th>Pixiv</th>
                        <th>Status</th>
                        <th>Image</th>
                      </tr>
                    </thead>
                    <tbody>
                      {task.items.map((item) => (
                        <tr key={item.item_id}>
                          <td>{item.item_id}</td>
                          <td>{item.pixiv_id || "-"}</td>
                          <td>{item.status}</td>
                          <td>{item.image_id || "-"}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <div className="log-list">
                  {task.logs.map((log) => (
                    <div className="log-line" key={log.log_id}>
                      <span>{log.level}</span>
                      <strong>{log.phase}</strong>
                      <p>{log.message}</p>
                    </div>
                  ))}
                </div>
              </>
            ) : null}

            {!task && !detailLoading ? (
              <p className="quiet">Paste a task id or open one from Recent Tasks.</p>
            ) : null}
          </section>
        </div>
      ) : null}
    </div>
  );
}
