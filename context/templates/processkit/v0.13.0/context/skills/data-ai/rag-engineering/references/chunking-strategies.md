# Chunking Strategies Reference

## Strategy Comparison

| Strategy | Best For | Chunk Quality | Implementation | Pitfall |
|----------|----------|---------------|----------------|---------|
| Fixed-size + overlap | Homogeneous text, fast baseline | Medium | Trivial | Splits mid-sentence |
| Sentence-based | Articles, documentation | Good | Easy | Uneven chunk sizes |
| Semantic | Mixed-topic documents | High | Moderate | Slow, embedding cost |
| Recursive character | General purpose | Good | Easy (LangChain) | Arbitrary split points |
| Document-structure | Well-formatted docs, code | High | Moderate | Depends on structure quality |

## Fixed-Size with Overlap

Split every N tokens. Overlap by M tokens to avoid losing context at boundaries.

```python
def fixed_size_chunks(text, chunk_size=512, overlap=50):
    tokens = tokenizer.encode(text)
    chunks = []
    for i in range(0, len(tokens), chunk_size - overlap):
        chunk_tokens = tokens[i:i + chunk_size]
        chunks.append(tokenizer.decode(chunk_tokens))
    return chunks
```

**When to use:** Quick baseline, uniform-length documents, when you need predictable chunk sizes for batching. Works well enough for many production systems.

**Parameters:** 256-1024 tokens chunk size. Overlap 10-20% of chunk size. Larger chunks for documents with long explanations; smaller for FAQ-style content.

## Sentence-Based

Split on sentence boundaries, then group sentences into chunks.

```python
from nltk.tokenize import sent_tokenize

def sentence_chunks(text, max_sentences=5, overlap_sentences=1):
    sentences = sent_tokenize(text)
    chunks = []
    for i in range(0, len(sentences), max_sentences - overlap_sentences):
        chunk = " ".join(sentences[i:i + max_sentences])
        chunks.append(chunk)
    return chunks
```

**When to use:** Prose-heavy documents where sentence boundaries are meaningful. News articles, blog posts, documentation.

**Watch out for:** Very short or very long sentences creating uneven chunks. Consider falling back to fixed-size if variance is too high.

## Semantic Chunking

Compute embeddings for each sentence (or small segment). Split where cosine similarity between consecutive segments drops below a threshold.

```python
import numpy as np
from sklearn.metrics.pairwise import cosine_similarity

def semantic_chunks(sentences, embeddings, threshold=0.75):
    chunks, current_chunk = [], [sentences[0]]
    for i in range(1, len(sentences)):
        sim = cosine_similarity(
            [embeddings[i-1]], [embeddings[i]]
        )[0][0]
        if sim < threshold:
            chunks.append(" ".join(current_chunk))
            current_chunk = [sentences[i]]
        else:
            current_chunk.append(sentences[i])
    chunks.append(" ".join(current_chunk))
    return chunks
```

**When to use:** Documents covering multiple topics in sequence. Research papers, long reports, meeting transcripts. When topic coherence within chunks matters more than uniform size.

**Cost:** Requires embedding every sentence at indexing time. For large corpora, use a fast local model (e.g., `all-MiniLM-L6-v2`) for chunking, then re-embed final chunks with a better model.

**Threshold tuning:** Start at 0.75. Lower (0.6) = fewer, larger chunks. Higher (0.85) = more, smaller chunks. Plot the similarity curve to find natural breakpoints.

## Recursive Character Splitting

Split on a hierarchy of separators: paragraphs first, then sentences, then words. LangChain's default approach.

```python
separators = ["\n\n", "\n", ". ", " ", ""]

def recursive_split(text, chunk_size=512, separators=separators):
    sep = separators[0]
    parts = text.split(sep)
    chunks, current = [], ""
    for part in parts:
        if len(current) + len(part) > chunk_size:
            if current:
                chunks.append(current.strip())
            if len(part) > chunk_size and len(separators) > 1:
                chunks.extend(recursive_split(part, chunk_size, separators[1:]))
                current = ""
            else:
                current = part
        else:
            current = current + sep + part if current else part
    if current:
        chunks.append(current.strip())
    return chunks
```

**When to use:** General-purpose default when you don't know document structure. Reasonable results across most document types.

**Customize separators for your format:** Markdown: `["\n## ", "\n### ", "\n\n", "\n", " "]`. Python: `["\nclass ", "\ndef ", "\n\n", "\n", " "]`.

## Document-Structure-Aware

Parse document structure (headings, sections, code blocks) and use it to define chunk boundaries.

```python
import re

def markdown_chunks(text, max_chunk_size=1024):
    # Split on headings, keep heading with content
    sections = re.split(r'(^#{1,3} .+$)', text, flags=re.MULTILINE)
    chunks, current = [], ""
    for section in sections:
        if len(current) + len(section) > max_chunk_size and current:
            chunks.append(current.strip())
            current = section
        else:
            current += "\n" + section
    if current.strip():
        chunks.append(current.strip())
    return chunks
```

**When to use:** Markdown docs, HTML with clear structure, code files, legal documents with numbered sections. Anywhere document structure maps to semantic boundaries.

**For code:** Use tree-sitter to parse AST. Chunk at function/class level. Include the import block and class signature as context prefix for each method chunk.

## Practical Guidelines

1. **Always measure.** Chunk strategy changes retrieval quality. Run eval after every change.
2. **Embed metadata in chunks.** Prepend `"Title: {title}\nSection: {heading}\n\n"` to each chunk. This improves retrieval when queries reference document titles or sections.
3. **Track provenance.** Store source file, page/line number, heading path with each chunk. Essential for citations and debugging.
4. **Small chunks for retrieval, large chunks for generation.** Use parent-document retrieval: embed at 256 tokens, but return the 1024-token parent when matched.
5. **Deduplication.** After chunking, remove near-duplicates (cosine similarity > 0.95). Common with overlapping chunks or repeated boilerplate.
