# Knovum search visibility plan

This file documents the search intent owned by each page. It is a content-maintenance map, not a keyword-stuffing checklist.

## Positioning

Knovum is a free, MIT-licensed, local-first desktop knowledge workspace for research, learning, and creation. It combines Spaces and an infinite canvas, mixed-media source material, local keyword and semantic search, source-linked answers, structured knowledge, and traceable creation.

The website should consistently communicate three things:

1. The product is a desktop knowledge workspace, not a hosted note-taking SaaS.
2. The knowledge library is local-first; cloud AI is optional and task-scoped.
3. Answers and derived work should remain connected to the sources and revisions that shaped them.

## Page-to-intent map

| Page | Primary search intent | Primary English query family | Chinese query family |
| --- | --- | --- | --- |
| `index.html` | Discover the product category | local-first knowledge management software; desktop AI research workspace; open-source knowledge workspace | 本地优先知识管理软件；桌面 AI 研究工作空间；开源知识管理工具 |
| `product.html` | Evaluate product capabilities | infinite canvas for research; research workspace with citations; semantic search for notes and PDFs; source-cited AI answers | 无限画布研究工具；带来源引用的研究工作空间；笔记与 PDF 语义搜索 |
| `privacy.html` | Understand data custody | private knowledge base for desktop; local-first AI workspace; keep research notes on device | 本地知识库；隐私知识管理软件；数据保存在本地的 AI 工作空间 |
| `download.html` | Install the product | Knovum download; free open-source macOS knowledge app; Windows knowledge workspace | Knovum 下载；macOS 知识管理软件；Windows 桌面知识库 |
| `about.html` | Establish project identity and trust | Knovum open source; local-first knowledge workspace project | Knovum 开源项目；本地优先知识工作空间 |
| `guides/local-first-knowledge-management.html` | Learn the category and trade-offs | what is local-first knowledge management; local-first vs cloud-first knowledge base | 什么是本地优先知识管理；本地知识库与云端知识库区别 |
| `guides/infinite-canvas-for-research.html` | Learn a research workflow | how to use an infinite canvas for research; visual research workspace | 如何用无限画布做研究；视觉化研究工作流 |
| `guides/source-grounded-ai-research.html` | Evaluate evidence quality | AI research with source citations; how to verify AI answers | AI 研究如何引用来源；如何核验 AI 答案 |
| `guides/semantic-search-for-notes-and-pdfs.html` | Understand retrieval | semantic search for notes and PDFs; keyword vs semantic search | 笔记与 PDF 语义搜索；关键词搜索与语义搜索区别 |

Keep each page focused on its assigned intent. Link related pages where it helps the reader continue, rather than repeating the same paragraph and heading across multiple URLs.

## Generative Engine Optimization (GEO) principles

GEO is treated as discoverability and citation readiness across search experiences, not as a separate markup trick.

- Keep essential content in crawlable HTML. English and Simplified Chinese pages are independently rendered as static HTML, so crawlers do not need to execute the language switch to discover either language.
- Start major sections with a concise, direct explanation, then give definitions, steps, limitations, and examples.
- Use descriptive headings that match real questions people ask. Keep factual claims close to their supporting context.
- Distinguish product facts from general advice. State what Knovum does, what is optional, what requires downloaded models, and what depends on a selected third-party provider.
- Keep source claims, platform requirements, version details, and supported formats aligned with the README, DESIGN.md, and technical documentation.
- Use real product images with descriptive alt text. Do not present generated UI mockups as shipped product features.
- Allow search crawlers such as OAI-SearchBot and Claude-SearchBot separately from foundation-model training crawlers. See `robots.txt` and the crawler documentation.
- Do not add `llms.txt`, hidden keyword blocks, fake author profiles, fabricated reviews, or mass-produced doorway pages as ranking shortcuts. Google does not require special GEO files or markup for AI features.

## Technical SEO checklist

The source tree already includes:

- Unique title and meta description per core page and guide.
- Responsive viewport and index/follow metadata for public pages.
- `noindex,follow` on both 404 pages; 404 URLs are excluded from the sitemap.
- Open Graph and Twitter summary metadata.
- SoftwareApplication structured data on the home page and visible FAQPage structured data on the download page.
- English and Simplified Chinese static routes.
- Internal links between product, privacy, download, and relevant guides.
- Search-crawler rules that allow search/grounding agents while separately controlling training crawlers.
- A link, anchor, metadata, asset, and accessibility checker in `scripts/check_site.py`.

### GitHub Pages transition

The transition hostname for this repository is `https://madebyzan.github.io/knovum/`. The `deploy-website.yml` workflow builds the sitemap and absolute canonical / Open Graph / hreflang tags for this project path, then deploys the `website/` directory to GitHub Pages after changes reach `main`. The repository must have **Settings → Pages → Build and deployment → Source = GitHub Actions** enabled once by a repository administrator.

GitHub Pages project sites are served below `/knovum/`, while the robots exclusion protocol is defined at the hostname root. Therefore, the project-scoped `website/robots.txt` is retained for the site and future custom-domain migration, but do not assume crawlers will use `/knovum/robots.txt` as the host-level robots policy. Submit `https://madebyzan.github.io/knovum/sitemap.xml` directly in Google Search Console and Bing Webmaster Tools during the transition.

The social preview artwork lives in `website/assets/social-card.svg` and `website/assets/social-card-zh.svg`. The workflow renders localized 1200×630 vector artwork to PNG before deployment; English pages use the English card and Simplified Chinese pages use the Chinese card. Open Graph and Twitter/X both point to broadly supported PNGs. The source SVGs remain editable in the repository. Once a custom domain is selected, change the `--base-url` in `.github/workflows/deploy-website.yml` and add the matching GitHub Pages custom-domain configuration / DNS records. Do not keep the GitHub Pages URL as canonical after the domain migration.

After the permanent production hostname is confirmed, run:

```bash
python3 website/scripts/generate_sitemap.py --base-url https://knovum.example --apply-html
```

This generates an absolute-URL XML sitemap, adds the sitemap directive to `robots.txt`, and writes canonical, Open Graph URL, and `hreflang` tags (en, zh-CN, x-default) to matching language pages. Replace the placeholder with the real HTTPS site origin before publishing.

## Measurement after launch

1. Verify the production domain in Google Search Console and submit `/sitemap.xml`.
2. Verify the same domain in Bing Webmaster Tools and submit the sitemap. Consider IndexNow after the production host and API key are configured.
3. Check crawl and indexing reports, canonical selection, language-page coverage, and search queries separately for English and Simplified Chinese.
4. Review landing-page engagement and download clicks. Keep analytics privacy-conscious and do not add third-party tracking without a deliberate product/privacy decision.
5. Update long-tail guides only when the product or the underlying guidance changes. Measure qualified visits and assisted downloads rather than chasing impressions alone.

## Geographic targeting

Knovum is a downloadable desktop product, not a local business with a service area or physical customer location. LocalBusiness markup and city-keyword pages would be misleading. The relevant locale strategy is English plus Simplified Chinese with `hreflang`, not invented geographic targeting.
