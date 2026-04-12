# Prompt Techniques Catalog

Each technique with template, when to use, and example prompt with response.

## 1. Zero-Shot Prompting

Direct instruction without examples. Relies on the model's pre-trained knowledge.

**When to use:** Simple, well-defined tasks where the model's default behavior is close to correct.

**Template:**
```
[Task instruction]
[Constraints]
[Input]
```

**Example:**
```
Classify the following customer review as positive, negative, or neutral.
Respond with only the classification label.

Review: "The shipping was slow but the product quality exceeded my expectations."
```
Response: `positive`

## 2. Few-Shot Prompting

Provide input/output examples to demonstrate the desired pattern.

**When to use:** When the task requires a specific format, style, or reasoning pattern that is hard to describe in words. When zero-shot output is inconsistent.

**Template:**
```
[Task instruction]

Example 1:
Input: [example input]
Output: [example output]

Example 2:
Input: [example input]
Output: [example output]

Now process:
Input: [actual input]
Output:
```

**Example:**
```
Extract the product name and price from the description.

Example 1:
Input: "Get the new AirPods Pro for just $249.99"
Output: {"product": "AirPods Pro", "price": 249.99}

Example 2:
Input: "Samsung Galaxy S24 Ultra now available at $1199"
Output: {"product": "Samsung Galaxy S24 Ultra", "price": 1199.00}

Now process:
Input: "Grab the Sony WH-1000XM5 headphones, only $348"
Output:
```
Response: `{"product": "Sony WH-1000XM5", "price": 348.00}`

Tips: Use 3-5 examples. Include edge cases. Order examples from simple to complex, with the most similar to the expected input last.

## 3. Chain-of-Thought (CoT)

Ask the model to show its reasoning step by step before answering.

**When to use:** Math, logic, multi-step reasoning, complex analysis. Any task where getting the right answer requires intermediate steps.

**Template (zero-shot CoT):**
```
[Question]

Let's think through this step by step.
```

**Template (few-shot CoT):**
```
[Task instruction]

Q: [example question]
A: Let's think step by step.
[step 1 reasoning]
[step 2 reasoning]
Therefore, the answer is [answer].

Q: [actual question]
A: Let's think step by step.
```

**Example:**
```
Q: A store has 45 apples. They sell 3/5 of them in the morning and half
of the remainder in the afternoon. How many apples are left?

A: Let's think step by step.
1. Starting apples: 45
2. Sold in morning: 45 * 3/5 = 27
3. Remaining after morning: 45 - 27 = 18
4. Sold in afternoon: 18 / 2 = 9
5. Remaining: 18 - 9 = 9
Therefore, 9 apples are left.
```

## 4. Self-Consistency

Generate multiple CoT responses and take the majority answer.

**When to use:** Tasks with a single correct answer where individual CoT attempts may err. Improves accuracy at the cost of multiple API calls.

**Process:**
1. Run the same CoT prompt 3-5 times with temperature 0.7-1.0
2. Extract the final answer from each response
3. Take the majority vote

Best for: math word problems, factual questions, logical puzzles.

## 5. Structured Output (JSON Mode)

Constrain the model to produce valid structured data.

**When to use:** Any task where output must be parsed programmatically.

**Template:**
```
Extract the following information and return as JSON.
Use exactly this schema (no additional fields):

{
  "field_name": "type — description",
  "field_name": "type — description"
}

If a field is not present in the input, use null.

Input:
---
[input text]
---
```

**Example:**
```
Extract event details from the text below. Return JSON matching this schema:
{
  "event_name": "string",
  "date": "YYYY-MM-DD or null",
  "location": "string or null",
  "attendee_count": "integer or null"
}

Text: "The annual DevConf will be held on March 15, 2026 at the
Convention Center in Berlin. We expect around 2,500 attendees."
```
Response:
```json
{
  "event_name": "DevConf",
  "date": "2026-03-15",
  "location": "Convention Center, Berlin",
  "attendee_count": 2500
}
```

Tips: Provide the exact schema. Use JSON mode API parameter when available. Validate output with a schema validator.

## 6. Persona / Role Prompting

Assign the model a specific expert role to influence response style and depth.

**When to use:** When domain expertise matters. When you want a specific tone or level of detail.

**Template:**
```
You are a [specific role] with [years] of experience in [domain].
Your communication style is [style description].

[Task instruction]
```

**Example:**
```
You are a senior security engineer with 15 years of experience
in application security. You communicate findings clearly and
prioritize by severity.

Review the following code snippet for security vulnerabilities.
For each issue, provide: severity (critical/high/medium/low),
the vulnerability type, and a fix.

```python
def login(username, password):
    query = f"SELECT * FROM users WHERE name='{username}' AND pass='{password}'"
    user = db.execute(query).fetchone()
    return user is not None
```​
```

## 7. Constrained Generation with Stop Sequences

Use stop sequences to control output length and format.

**When to use:** Extracting a single piece of information. Preventing runaway generation.

**Implementation:**
```python
response = client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": prompt}],
    stop=["\n\n", "---"],  # stop at double newline or separator
    max_tokens=100
)
```

## 8. Prompt Chaining

Break complex tasks into sequential prompts, where each output feeds the next.

**When to use:** Multi-step tasks that are too complex for a single prompt. When intermediate results need validation.

**Process:**
```
Prompt 1: Extract key facts from document -> facts
Prompt 2: Given these facts, identify contradictions -> issues
Prompt 3: Given facts and issues, write summary -> final output
```

Each step can use a different model, temperature, or prompt strategy. Validate intermediate outputs before passing to the next step.
