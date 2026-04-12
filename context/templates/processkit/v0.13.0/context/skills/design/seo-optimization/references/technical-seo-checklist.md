# Technical SEO Checklist

## Crawling and Indexing

- [ ] `robots.txt` exists at site root and allows important paths
- [ ] `robots.txt` blocks admin, staging, and duplicate content paths
- [ ] XML sitemap exists and lists all indexable pages
- [ ] Sitemap excludes `noindex` pages, redirects, and error pages
- [ ] Sitemap submitted to Google Search Console and Bing Webmaster Tools
- [ ] No orphan pages — all indexable pages reachable via internal links
- [ ] Redirect chains resolved (max 1 hop)
- [ ] No redirect loops
- [ ] 404 pages return proper 404 status (not soft 404s)
- [ ] Removed pages return 410 (Gone) or redirect to relevant content

## On-Page Elements

- [ ] Unique `<title>` on every page (50-60 characters)
- [ ] Unique `<meta name="description">` on every page (120-160 characters)
- [ ] One `<h1>` per page, matching primary topic
- [ ] Heading hierarchy is logical (`h1` > `h2` > `h3`, no skipping)
- [ ] All images have descriptive `alt` attributes
- [ ] Decorative images use `alt=""`
- [ ] Internal links use descriptive anchor text
- [ ] URLs are clean, lowercase, hyphen-separated

## Canonical and Duplicate Content

- [ ] Every page has `<link rel="canonical" href="...">` pointing to itself or preferred version
- [ ] Pagination uses `rel="canonical"` to the appropriate page (not all to page 1)
- [ ] URL parameters do not create duplicate pages (or are handled in Search Console)
- [ ] WWW vs non-WWW redirects configured (one canonical version)
- [ ] Trailing slash behavior is consistent (with or without, pick one)
- [ ] HTTP redirects to HTTPS

## Performance (Core Web Vitals)

- [ ] LCP < 2.5 seconds (measure with Lighthouse or CrUX)
- [ ] INP < 200ms
- [ ] CLS < 0.1
- [ ] Images have explicit `width` and `height` attributes
- [ ] Above-fold images are preloaded; below-fold images use `loading="lazy"`
- [ ] Fonts preloaded with `font-display: swap`
- [ ] Critical CSS inlined; non-critical CSS deferred
- [ ] JavaScript deferred or loaded async where possible
- [ ] Brotli or gzip compression enabled for text resources
- [ ] Static assets served from CDN

## Structured Data

- [ ] JSON-LD format used (not Microdata or RDFa)
- [ ] `@context` set to `https://schema.org`
- [ ] Most specific `@type` used (e.g., `Product` not `Thing`)
- [ ] All structured data fields match visible page content
- [ ] Validated with Google Rich Results Test (no errors)
- [ ] BreadcrumbList added for hierarchical pages

### Common Schema Types

```json
{
  "@context": "https://schema.org",
  "@type": "Article",
  "headline": "Article Title",
  "author": { "@type": "Person", "name": "Author Name" },
  "datePublished": "2025-01-15",
  "image": "https://example.com/image.jpg"
}
```

```json
{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "Product Name",
  "image": "https://example.com/product.jpg",
  "description": "Product description",
  "offers": {
    "@type": "Offer",
    "price": "29.99",
    "priceCurrency": "USD",
    "availability": "https://schema.org/InStock"
  }
}
```

```json
{
  "@context": "https://schema.org",
  "@type": "FAQPage",
  "mainEntity": [{
    "@type": "Question",
    "name": "What is the return policy?",
    "acceptedAnswer": {
      "@type": "Answer",
      "text": "You can return items within 30 days."
    }
  }]
}
```

## Mobile Optimization

- [ ] `<meta name="viewport" content="width=device-width, initial-scale=1">`
- [ ] Base font size at least 16px
- [ ] No horizontal scrolling at any viewport width
- [ ] Touch targets at least 48x48 CSS pixels
- [ ] Content parity between mobile and desktop versions
- [ ] Passes Google Mobile-Friendly Test
- [ ] No intrusive interstitials blocking content on mobile

## Security

- [ ] All pages served over HTTPS
- [ ] HSTS header configured with long max-age
- [ ] No mixed content (HTTP resources on HTTPS pages)
- [ ] Security headers set: `X-Content-Type-Options`, `X-Frame-Options`
- [ ] CSP (Content Security Policy) configured

## International (if applicable)

- [ ] `hreflang` tags on all language/region variants
- [ ] Self-referencing `hreflang` included
- [ ] `x-default` hreflang set for language selector page
- [ ] Language-specific sitemaps or sitemap index
