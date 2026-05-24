Pixiv AI Downloader & Smart Retrieval Platform (Personal Edition)

1. Project Overview

Name: Pixiv AI Downloader (Personal Edition)

Objective: Provide an efficient, intelligent, and visual Pixiv image download and management tool that supports downloads by ID, author, bookmarks, daily top 10, and natural language-driven smart retrieval. Supports category management, map-based search, and optional NSFW/R18 control.

Tech Stack:

* Backend: Rust + Axum + Tokio (async tasks)
* Frontend: Next.js + CSS
* AI: DeepSeek V4 (base_url=https://api.deepseek.com)
* Storage: Local filesystem + SQLite indexing (manage categories, tags, map coordinates, deduplication, download logs)

⸻

2. Functional Modules

2.1 Pixiv Download Module

Feature	Description
Download Single Work	Input Work ID to download the original image
Download Bookmarks	Default: download user bookmarks with quantity & category limits
Download Author Works	Input author UID, control number & category
Daily Top10	Fetch Pixiv daily top 10 works, display in carousel
Random Surprise	Randomly download a work
Deduplication	All images deduplicated by Pixiv ID
Download Task Queue	Async processing with frontend polling

Note: Cookie input (PHPSESSID) is manual. No automatic refresh in V1.

⸻

2.2 Smart Retrieval Module

Feature	Description
Natural Language Search	User input analyzed by LLM to generate tags + count_recommend + optional negative tags + NSFW/R18
Tag-based Search	Pull images from Pixiv based on LLM-generated tags
Quantity Control	User sets number of images to download
Category Option	NSFW/R18 selectable
Result Display	Shared display module with other downloaded images, supports lazy load & map-based search

⸻

2.3 Image Management Module

Feature	Description
Category Management	SQLite stores categories, tags, author UID, Pixiv ID, download time
Map-based Search	Visual map for tags or style heatmap
Deduplication	Unique Pixiv ID constraint
NSFW/R18 Optional	User selects visibility and download
Display	Waterfall + carousel + map-based combination, aesthetic-first design

⸻

2.4 Frontend Display Module

Feature	Description
Homepage	Daily Top10 carousel + Random Surprise + Category entry
Download Page	Download by ID / Bookmarks / Author / Smart retrieval
Task Panel	Async task visualization, status & logs
Settings Page	Cookie input, download path, DeepSeek config, theme selection
Gallery Page	Waterfall layout, tag/map search, category filters, click-to-zoom

⸻

2.5 System Configuration Module

Feature	Description
Cookie Input	Manual PHPSESSID
Download Path	Default ~/pixiv_downloads/
AI Settings	DeepSeek API key & base_url
Download Limits	Frontend sets max quantity per request
Category Controls	NSFW/R18 selectable
Daily Top Refresh	Manual refresh, daily auto-cache optional

⸻

3. Storage Design (Local + SQLite)

Table	Fields	Description
images	id (Pixiv ID)	Unique
images	author_uid	Author UID
images	tags	JSON array
images	category	Normal / R18 / NSFW
images	download_time	Timestamp
images	local_path	Local file path
images	source	bookmark / author / top10 / random / smart
images	map_coordinates	Optional map heatmap coords
tasks	task_id	Async task ID
tasks	type	Single / Bookmark / Author / Top10 / Random / Smart
tasks	status	pending / running / completed / failed
tasks	progress	Download progress
tasks	created_at	Creation timestamp
tasks	finished_at	Completion timestamp

⸻

4. Frontend Page Structure

Homepage
 ├─ Daily Top10 Carousel
 ├─ Random Surprise
 ├─ Category Entry
Download Page
 ├─ By ID
 ├─ Bookmarks
 ├─ Author
 ├─ Smart Retrieval
Task Panel
 ├─ Async Queue Visualization
 ├─ Status / Logs
Settings Page
 ├─ Cookie Input
 ├─ Download Path
 ├─ DeepSeek Config
 ├─ Theme Selection
Gallery Page
 ├─ Waterfall Layout
 ├─ Tag / Map Search
 ├─ Category Filter
 ├─ Click-to-Zoom

⸻

5. Download Task Flow (Async Queue)

User Submits Task
      ↓
Axum Receives Request → Create Task → Insert into tasks Table
      ↓
Tokio Worker Async Downloads → Save Local Path → Update images Table
      ↓
Frontend Polls Task Status → Waterfall / Carousel Display

⸻

6. UX & Aesthetic Design

* Waterfall layout + Top10 carousel
* Lazy load for performance
* Map-based tag visualization
* Theme preview: Sakura Light demo option plus one-click switch from the default theme
* Real-time task progress and status

⸻

7. MVP Scope (V1)

1. Single Work / Bookmark / Author download (local storage)
2. Pixiv Top10 display & download
3. Random download feature
4. Async download queue + status panel
5. Smart retrieval (DeepSeek V4)
6. Waterfall layout + lazy load
7. Category management + map index
8. Optional NSFW/R18
9. Deduplication by Pixiv ID

Notes: Baidu Cloud integration, Docker, and account system are marked as TBD / V2 enhancements.
