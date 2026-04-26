---
name: ai-fundamentals
description: >
  Explain and apply core ML/AI concepts — model types, training
  pipelines, evaluation metrics, and neural architectures.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-ai-fundamentals
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: data-ai
---

# AI Fundamentals

## Intro

The core ideas a practitioner needs to reason about machine learning:
the four learning paradigms, the standard model families, the training
pipeline order, the right metrics for the task, and the failure modes
that bite every project. Start from the data and the problem; pick the
model last.

## Overview

### Learning paradigms

There are four major paradigms. Pick from the data, not the model.

- **Supervised** — labeled input/output pairs; classification and
  regression.
- **Unsupervised** — no labels; clustering, dimensionality reduction,
  anomaly detection.
- **Reinforcement** — agent learns from a reward signal; sequential
  decision-making, robotics, game play.
- **Self-supervised** — labels come from the data itself (masked
  language modeling, contrastive learning); powers modern foundation
  models.

### Model families

Match complexity to the problem. Start simple.

- **Linear models** (linear/logistic regression, SVM) — interpretable,
  fast, surprisingly strong baselines.
- **Tree-based** (random forest, gradient boosting, XGBoost,
  LightGBM) — top performers on tabular/structured data; robust to
  feature scaling.
- **Neural networks** — best on unstructured data (images, text,
  audio); demand more data and compute.
- **Probabilistic** (naive Bayes, Gaussian processes, Bayesian
  networks) — useful when you need uncertainty quantification.

### Training pipeline order

A correct pipeline is always: clean and prepare data, **then split**,
then any preprocessing that uses statistics from data, then feature
engineering, then training, then hyperparameter tuning against the
validation set, then a single final evaluation on the held-out test
set. Tuning against the test set is a bug, not a step.

Use regularization (L1, L2, dropout, early stopping) to control
overfitting. Use stratified k-fold cross-validation for small or
imbalanced datasets.

### Evaluation metrics

Choose metrics that match the business problem, not the metric your
framework defaults to.

| Task | Metrics | When to prefer |
|---|---|---|
| Binary classification | Accuracy, Precision, Recall, F1, AUC-ROC | F1 for imbalanced classes; AUC-ROC for ranking |
| Multi-class | Macro/micro F1, confusion matrix | Macro F1 when all classes matter equally |
| Regression | RMSE, MAE, R-squared | MAE when outliers shouldn't dominate; RMSE penalizes large errors |
| Ranking/retrieval | Precision@k, NDCG, MAP | NDCG when position matters |
| Generation (NLP) | BLEU, ROUGE, perplexity, human eval | Human eval is gold for open-ended generation |

Always report variance across folds or confidence intervals, not
single-point estimates.

### Neural architectures at a glance

- **MLP** — fully connected; general-purpose, no structural bias.
- **CNN** — spatial hierarchy via convolutions; images, time series.
- **RNN/LSTM/GRU** — sequential with memory; largely superseded by
  Transformers in NLP.
- **Transformer** — self-attention over all positions; dominant in
  NLP, vision (ViT), and multimodal.
- **GAN** — generator vs. discriminator; image synthesis.
- **VAE** — probabilistic encoder/decoder with structured latent
  space.
- **Diffusion** — iterative denoising; state of the art for image and
  video generation.

### Modern LLM concepts

Attention mechanism, subword tokenization (BPE/WordPiece/SentencePiece),
pre-training plus fine-tuning, RLHF (and now DPO) for preference
alignment, prompting strategies (zero-shot, few-shot, chain-of-thought,
RAG), and scaling laws (more data + parameters + compute, balanced
per Chinchilla-style ratios).

### Common pitfalls

- **Data leakage** — test info bleeds into training (normalizing
  before splitting, using future data).
- **Overfitting** — large train/test gap; fix with regularization,
  more data, simpler model.
- **Underfitting** — model too simple; add capacity or features.
- **Class imbalance** — stratified splits, class weights, SMOTE,
  focal loss.
- **Distribution shift** — train and prod diverge; monitor and
  retrain.
- **Metric mismatch** — optimizing something that doesn't reflect
  business value.
- **Confirmation bias in evaluation** — cherry-picking on
  non-representative slices.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Data leakage through preprocessing before the split.** Fitting a scaler, imputer, or encoder on the full dataset before splitting means test statistics influence the training fit — the test set is no longer held out. Always split first, then fit transformers on training data only.
- **Evaluating hyperparameters on the test set.** Tuning hyperparameters against the test set makes the test set part of the training process. Use a validation set or nested cross-validation; reserve the test set for a single final evaluation.
- **Choosing metrics based on framework defaults, not business goals.** A model optimized for accuracy on an imbalanced dataset can be 95% accurate by predicting the majority class on every sample. Choose metrics that match the actual cost of errors: F1 for imbalanced classes, AUC-ROC for ranking, RMSE vs. MAE depending on whether large errors matter disproportionately.
- **Shuffling a time series before splitting.** A shuffled time-series train/test split leaks future data into training. Use a time-based split where all training data precedes all test data, and never use cross-validation without a time-aware splitter.
- **Reporting only a single point estimate without variance.** A model that is 90% accurate on one fold may be 78% on another. Single-number metrics hide variance. Always report variance across folds or confidence intervals, not just the mean.
- **Skipping a baseline model.** Starting with a complex neural network when a logistic regression or a mean predictor hasn't been tried yet means you don't know how much complexity you actually need. Build and evaluate a simple baseline before adding sophistication.
- **Treating distribution shift as a training problem.** When production inputs drift from the training distribution (different season, different user base, different data pipeline), retraining on more of the original data won't fix it. Detect shift explicitly via feature distribution monitoring and collect representative production data for retraining.

## Full reference

### Bias-variance decomposition

Every model error decomposes:

```
Total error = Bias^2 + Variance + Irreducible noise
```

- High bias (underfitting) — simple model, both train and test error
  high.
- High variance (overfitting) — complex model, low train error but
  high test error.
- Diagnose with learning curves (error vs. training set size).

### Cross-validation variants

- **k-fold** — typical k=5 or 10; average metrics across folds.
- **Stratified k-fold** — preserves class distribution; essential for
  imbalanced data.
- **Leave-one-out** — k = n; low bias, high variance, expensive.
- **Time-series split** — expanding or sliding window; respects
  temporal order. Never shuffle time series.
- **Group k-fold** — keeps related samples (same patient, same user)
  in the same fold.
- **Nested CV** — outer loop evaluates, inner loop tunes; prevents
  optimistic bias from tuning on the eval set.

### Ensembles

- **Bagging** — train on bootstrap samples, average predictions;
  reduces variance. Random Forest is the canonical example.
- **Boosting** — train sequentially, each model corrects predecessors;
  reduces bias. XGBoost, LightGBM, CatBoost. More overfit-prone than
  bagging — control with learning rate, depth, early stopping.
- **Stacking** — train diverse base models, then a meta-model on
  their outputs.

### Gradient descent and optimizers

Update rule: `params = params - lr * grad(loss, params)`.

| Variant | Batch | Trade-off |
|---|---|---|
| Batch GD | Entire dataset | Stable but slow, high memory |
| SGD | Single sample | Noisy, fast, can escape local minima |
| Mini-batch | 32–512 | Standard for deep learning |

Optimizers: SGD with momentum, AdaGrad (sparse features), RMSProp,
Adam (default for most deep learning), AdamW (preferred for
Transformers — decoupled weight decay).

Learning rate is the most important hyperparameter. Use schedulers
(cosine, warmup + decay, reduce-on-plateau) and a learning-rate
finder.

### Backpropagation gotchas

Backprop is the chain rule applied layer by layer. Common issues:

- **Vanishing gradients** in deep nets — fix with ReLU, residual
  connections, careful init (Kaiming/Xavier).
- **Exploding gradients** — fix with gradient clipping or batch
  normalization.
- **Dead ReLU neurons** — use Leaky ReLU or ELU.

### Transfer learning

Reuse a model trained on one task for another. Lower layers learn
general features (edges, syntax); upper layers learn task-specific
ones.

- **Feature extraction** — freeze pre-trained layers, train a new
  head; fast and works on small datasets.
- **Fine-tuning** — unfreeze some/all layers, small learning rate;
  better with enough data.
- **Progressive unfreezing** — unfreeze top to bottom; reduces
  catastrophic forgetting.
- **Parameter-efficient methods** (LoRA, adapters) modify only a
  small fraction of weights.

Transfer can fail when source and target domains are too dissimilar,
when the target dataset is large enough to train from scratch, or
when pre-trained features actively hurt (negative transfer).

### Math foundations to internalize

- **Linear algebra** — vectors as features, matrices as
  transformations, dot product as similarity, eigenvectors as natural
  axes (PCA = top eigenvectors of covariance), SVD generalizes to
  non-square matrices and powers low-rank approximations like LoRA.
- **Probability** — Bayes' theorem (prior, likelihood, posterior),
  the standard distributions (Bernoulli, Categorical, Gaussian,
  Poisson, Multinomial), MLE (cross-entropy is negative log
  likelihood; MSE is MLE under Gaussian noise).
- **Calculus** — gradients point in the direction of steepest ascent;
  the chain rule is the engine of backprop; the Jacobian generalizes
  the gradient to vector outputs.
- **Information theory** — entropy `H(X) = -sum P log P` measures
  uncertainty; cross-entropy is the standard classification loss;
  KL divergence measures how different two distributions are (used
  in VAEs, distillation, Bayesian regularization); mutual information
  drives feature selection and contrastive learning (InfoNCE).

For deeper treatment of cross-validation, gradient descent variants,
backprop, transfer learning, and ensembles, see
`references/ml-concepts.md`. For the math derivations and worked
examples, see `references/math-foundations.md`.

### Worked scenarios

**Tabular customer churn (50K rows).** Start with gradient-boosted
trees (XGBoost or LightGBM). Use stratified k-fold because churn is
typically 5–10% positive. Report F1 and AUC-ROC. Baseline with
logistic regression first. Only consider neural approaches if trees
plateau and you have far more data.

**Reviewing a pipeline for leakage.** A teammate normalizes the full
dataset before splitting train/test. This leaks test statistics.
Fix: split first, fit the scaler on train only, transform both with
the train scaler. Same applies to target encoding, PCA, and any
transformation that learns from data.

**Document Q&A.** Use a Transformer-based RAG pattern: retrieve
relevant passages with a vector store, feed them as context to an
LLM. Fine-tune the retriever on domain queries if generic embeddings
underperform. Evaluate with exact match, F1 over answer tokens, and
human judgment on a held-out set. Monitor hallucination by checking
answer grounding against retrieved passages.
