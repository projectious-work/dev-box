# Math Foundations for ML

Essential mathematics with intuitive explanations. Focus on what each concept means for ML, not formal proofs.

## Linear Algebra

### Vectors

A vector is a point or direction in n-dimensional space. In ML, a data sample is a vector of features.

- **Dot product** — `a . b = sum(a_i * b_i)` = measures similarity between vectors; cosine similarity normalizes by magnitude
- **Norm** — `||x||_2 = sqrt(sum(x_i^2))` = length/magnitude of a vector; L1 norm (`sum(|x_i|)`) encourages sparsity
- **Unit vector** — `x / ||x||` = direction without magnitude; used in normalization

### Matrices

A matrix is a linear transformation. In ML, the data matrix is (samples x features), weight matrices define layer transformations.

- **Matrix multiplication** — `(m x n) @ (n x p) -> (m x p)` = composing transformations; a neural network layer is `output = activation(W @ input + b)`
- **Transpose** — `A^T` = flip rows and columns; `(AB)^T = B^T A^T`
- **Inverse** — `A^{-1}` such that `A @ A^{-1} = I`; exists only for square, non-singular matrices; used in closed-form linear regression: `w = (X^T X)^{-1} X^T y`
- **Rank** — number of linearly independent rows/columns; low-rank matrices can be compressed (LoRA exploits this)

### Eigenvalues and Eigenvectors

`A @ v = lambda * v` — the eigenvector v is only scaled (not rotated) by the transformation A.

- **Intuition** — eigenvectors are the "natural axes" of the transformation
- **PCA** — finds eigenvectors of the covariance matrix; the top eigenvectors capture directions of maximum variance
- **Spectral methods** — graph Laplacian eigenvectors for clustering, spectral normalization for GANs
- **Singular Value Decomposition (SVD)** — `A = U S V^T`; generalizes eigendecomposition to non-square matrices; used in dimensionality reduction, matrix completion, low-rank approximations

## Probability and Statistics

### Bayes' Theorem

```
P(A|B) = P(B|A) * P(A) / P(B)
```

- **Prior** `P(A)` — belief before seeing data
- **Likelihood** `P(B|A)` — how probable the data is under hypothesis A
- **Posterior** `P(A|B)` — updated belief after seeing data
- **ML application** — naive Bayes classifiers, Bayesian optimization for hyperparameter search, Bayesian neural networks for uncertainty

### Key Distributions

| Distribution | Type | ML use |
|-------------|------|--------|
| Bernoulli | Discrete, binary | Binary classification output |
| Categorical | Discrete, k classes | Multi-class softmax output |
| Gaussian (Normal) | Continuous, bell curve | Noise modeling, weight initialization, VAE latent space |
| Uniform | Continuous, flat | Random initialization, exploration in RL |
| Poisson | Discrete, count data | Event count modeling |
| Multinomial | Discrete, counts over k classes | Text modeling (bag of words) |

### Maximum Likelihood Estimation (MLE)

Find parameters that maximize `P(data | parameters)`.

- Equivalent to minimizing negative log-likelihood
- Cross-entropy loss = negative log-likelihood for classification
- MSE loss = MLE under Gaussian noise assumption for regression

### Expectation and Variance

- **E[X]** = weighted average of outcomes = `sum(x * P(x))` = the "center" of the distribution
- **Var(X)** = `E[(X - E[X])^2]` = spread around the mean; high variance means unpredictable
- **Law of large numbers** — sample mean converges to E[X]; justifies using mini-batch averages as gradient estimates

## Calculus

### Gradients

The gradient `nabla f(x)` is a vector of partial derivatives pointing in the direction of steepest ascent.

- **Scalar function, vector input** — `nabla f = [df/dx_1, df/dx_2, ..., df/dx_n]`
- **Negative gradient** — direction of steepest descent; this is what gradient descent follows
- **Gradient magnitude** — how steep the function is; near a minimum, gradients are small

### Chain Rule

For composed functions `f(g(x))`:

```
df/dx = (df/dg) * (dg/dx)
```

- **Why it matters** — backpropagation is just the chain rule applied repeatedly through network layers
- **Multi-variable** — for `f(g(x), h(x))`, sum contributions: `df/dx = (df/dg)(dg/dx) + (df/dh)(dh/dx)`
- **Jacobian** — matrix of all partial derivatives for vector-valued functions; generalizes gradient to vector outputs

### Practical Implications

- **Vanishing gradients** — repeated multiplication of small values (<1) through many layers; sigmoid/tanh saturate, producing near-zero gradients
- **Exploding gradients** — repeated multiplication of large values; gradient clipping caps the norm
- **Second-order methods** — use curvature (Hessian) for better updates; too expensive for large nets but approximations (Adam, L-BFGS) help

## Information Theory

### Entropy

```
H(X) = -sum(P(x) * log P(x))
```

- **Intuition** — measures uncertainty or "surprise" in a distribution
- **Low entropy** — distribution is peaked, outcomes are predictable
- **High entropy** — distribution is flat, outcomes are uncertain
- **Binary entropy** — for a coin with bias p: `H = -p log(p) - (1-p) log(1-p)`; maximized at p=0.5
- **ML use** — decision tree splitting criterion; higher entropy = more information gain from a split

### Cross-Entropy

```
H(P, Q) = -sum(P(x) * log Q(x))
```

- **Intuition** — cost of encoding data from distribution P using a code optimized for distribution Q
- **Always >= entropy** — `H(P, Q) >= H(P)`, with equality when Q = P
- **ML use** — the standard classification loss; P is the true label distribution (one-hot), Q is the model's predicted probabilities
- **Binary cross-entropy** — `-(y log(p) + (1-y) log(1-p))` for binary classification

### KL Divergence

```
D_KL(P || Q) = sum(P(x) * log(P(x) / Q(x))) = H(P, Q) - H(P)
```

- **Intuition** — measures how different Q is from P; the "extra cost" of using Q instead of P
- **Not symmetric** — `D_KL(P || Q) != D_KL(Q || P)`
- **Always >= 0** — equals zero only when P = Q
- **ML use** — VAE loss (KL between encoder output and prior), knowledge distillation (match student to teacher distribution), regularization in Bayesian methods

### Mutual Information

```
I(X; Y) = H(X) - H(X|Y) = D_KL(P(X,Y) || P(X)P(Y))
```

- **Intuition** — how much knowing Y reduces uncertainty about X
- **ML use** — feature selection (select features with highest MI to target), representation learning (InfoNCE loss in contrastive learning)
