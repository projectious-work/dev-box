# ML Concepts Reference

## Learning Paradigms

### Supervised Learning

Model learns a mapping f(x) -> y from labeled examples.

- **Classification** — discrete output: spam detection, image recognition, medical diagnosis
- **Regression** — continuous output: price prediction, demand forecasting, age estimation

### Unsupervised Learning

Model discovers structure in unlabeled data.

- **Clustering** — k-means, DBSCAN, hierarchical; customer segmentation, topic discovery
- **Dimensionality reduction** — PCA, t-SNE, UMAP; visualization, noise removal, feature compression
- **Anomaly detection** — isolation forest, autoencoders; fraud detection, manufacturing defects

### Reinforcement Learning

Agent takes actions in an environment to maximize cumulative reward.

- **Key components** — state, action, reward, policy, value function
- **Exploration vs. exploitation** — balance trying new actions with exploiting known good ones
- **Applications** — game playing (AlphaGo), robotics, recommendation systems, RLHF for LLMs
- **Algorithms** — Q-learning, policy gradient, PPO, SAC

## Bias-Variance Tradeoff

Every model error decomposes into three parts:

```
Total Error = Bias^2 + Variance + Irreducible Noise
```

- **High bias (underfitting)** — model is too simple, consistently wrong in the same direction; training and test error are both high
- **High variance (overfitting)** — model is too complex, fits noise; training error is low but test error is high
- **Sweet spot** — enough complexity to capture real patterns without fitting noise

Practical diagnostics:
- Plot learning curves (error vs. training set size)
- If train error is low but val error is high: reduce complexity, add regularization, get more data
- If both errors are high: increase complexity, add features, reduce regularization

## Cross-Validation

Reduces evaluation variance by using all data for both training and validation.

### k-Fold Cross-Validation

1. Split data into k equal folds (typically k=5 or k=10)
2. For each fold: train on k-1 folds, validate on the held-out fold
3. Average metrics across all k runs; report mean and standard deviation

### Variants

- **Stratified k-fold** — preserves class distribution in each fold; essential for imbalanced datasets
- **Leave-one-out (LOO)** — k = n; low bias but high variance and expensive
- **Time-series split** — expanding or sliding window; respects temporal ordering
- **Group k-fold** — ensures related samples (e.g., same patient) stay in the same fold

### Nested Cross-Validation

Use for unbiased model selection + evaluation:
- Outer loop: evaluate final performance
- Inner loop: tune hyperparameters
- Prevents optimistic bias from tuning on the same validation set used for evaluation

## Ensemble Methods

Combine multiple models to improve performance.

### Bagging (Bootstrap Aggregating)

- Train multiple models on bootstrap samples of the data
- Aggregate predictions (majority vote or average)
- Reduces variance; Random Forest is the canonical example
- Each tree sees a random subset of features at each split

### Boosting

- Train models sequentially, each correcting the errors of its predecessor
- Reduces bias; AdaBoost, Gradient Boosting, XGBoost, LightGBM, CatBoost
- More prone to overfitting than bagging; control with learning rate, tree depth, early stopping

### Stacking

- Train diverse base models, then train a meta-model on their outputs
- Can combine fundamentally different model types (trees + linear + neural)

## Gradient Descent

Iteratively minimize a loss function by following its gradient.

### Core Algorithm

```
parameters = parameters - learning_rate * gradient(loss, parameters)
```

### Variants

| Variant | Batch size | Trade-off |
|---------|-----------|-----------|
| Batch GD | Entire dataset | Stable but slow, high memory |
| Stochastic GD (SGD) | Single sample | Noisy but fast, can escape local minima |
| Mini-batch GD | Typically 32-512 | Best of both; standard in deep learning |

### Optimizers

- **SGD with momentum** — accumulates velocity to dampen oscillations
- **AdaGrad** — per-parameter learning rates; good for sparse features
- **RMSProp** — fixes AdaGrad's diminishing learning rate
- **Adam** — combines momentum + RMSProp; default choice for most deep learning
- **AdamW** — Adam with decoupled weight decay; preferred for Transformer training

### Learning Rate

Most important hyperparameter. Too high: diverges. Too low: slow convergence.
- Use learning rate schedulers: cosine annealing, warmup + decay, reduce on plateau
- Learning rate finder: sweep from small to large, pick the steepest descent point

## Backpropagation Intuition

How neural networks learn — computing gradients efficiently via the chain rule.

1. **Forward pass** — input flows through layers, producing a prediction and a loss value
2. **Backward pass** — loss gradient flows backward through the network via the chain rule
3. **Each layer** receives the gradient from the layer above and computes:
   - Gradient w.r.t. its parameters (for weight updates)
   - Gradient w.r.t. its input (to pass to the layer below)
4. **Chain rule** makes this efficient: the gradient at any layer is a product of local gradients along the path

### Common Issues

- **Vanishing gradients** — gradients shrink exponentially in deep networks; fix with ReLU, residual connections, careful initialization
- **Exploding gradients** — gradients grow exponentially; fix with gradient clipping, batch normalization
- **Dead neurons** — ReLU outputs zero for all inputs; use Leaky ReLU or ELU

## Transfer Learning

Reuse a model trained on one task for a different (related) task.

### Why It Works

Lower layers learn general features (edges, syntax), upper layers learn task-specific features. General features transfer across tasks.

### Strategies

1. **Feature extraction** — freeze pre-trained layers, train only a new head; fast, works with small datasets
2. **Fine-tuning** — unfreeze some or all layers, train with a small learning rate; better performance with enough data
3. **Progressive unfreezing** — unfreeze layers gradually from top to bottom; reduces catastrophic forgetting

### Modern Transfer Learning

- **NLP** — pre-trained LLMs (GPT, BERT) fine-tuned on downstream tasks; parameter-efficient methods (LoRA, adapters) modify only a small fraction of weights
- **Vision** — ImageNet-pretrained CNNs/ViTs as backbones for detection, segmentation, medical imaging
- **Multimodal** — CLIP-style models transfer across vision and language tasks

### When Transfer Learning Fails

- Source and target domains are too different (e.g., medical images from ImageNet features)
- Target dataset is large enough that training from scratch matches or exceeds transfer
- Negative transfer: pre-trained features hurt performance; diagnose by comparing with random initialization
