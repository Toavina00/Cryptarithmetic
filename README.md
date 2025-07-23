# Cryptarithmetic Problem

Cryptarithmetic, also known as verbal arithmetic, alphametics, cryptarithms, or word addition, is a type of mathematical puzzle involving an arithmetic equation where digits are represented by letters of the alphabet. The objective is to determine the numerical value of each letter to make the equation valid. [Wikipedia](https://en.wikipedia.org/wiki/Verbal_arithmetic)

In these puzzles, each letter typically corresponds to a unique digit, and the leading digit of any multi-digit number must not be zero, consistent with standard arithmetic notation. [Wikipedia](https://en.wikipedia.org/wiki/Verbal_arithmetic)

This solver models the cryptarithmetic problem as a Constraint Satisfaction Problem (CSP) and uses arc consistency to compute the solution.

## Inputs

The program accepts three words as input, where the sum of the first two words equals the third. For example, consider the words `SEND`, `MORE`, and `MONEY`, arranged as follows:

```
    SEND
  + MORE
  ------
   MONEY
```

## Variables

The primary variables are the unique letters from the input words, denoted as $ X_1, X_2, \dots, X_n $, where $ n $ is the total number of distinct letters across all three words.

To facilitate solving the problem, additional variables are introduced:

- **Carry variables**: $ C_1, C_2, \dots, C_d $, representing the carry value for each column in the addition, where $ d $ is the length of the longest input word.
- **Padding variable**: $ P $, used to standardize the lengths of the input words for computational convenience.
- **Hidden variables**: $ U_1, U_2, \dots, U_d $, introduced to binarize the problem. Each $ U_k $ combines three letter or padding variables ($ A_1, A_2, A_3 \in \{P\} \cup \{ X_1, X_2, \dots, X_n \} $) and two carry variables ($ B_1, B_2 \in \{ C_1, C_2, \dots, C_d \} $), constrained by the addition rule: $ B_1 + A_1 + A_2 = 10 B_2 + A_3 $.

## Domains

- Each letter variable $ X_i $ has the domain $ \{0, 1, 2, 3, 4, 5, 6, 7, 8, 9\} $.
- Each carry variable $ C_i $ typically has the domain $ \{0, 1\} $.
- The padding variable $ P $ is fixed to the domain $ \{0\} $.
- Each hidden variable $ U_k $ has a domain derived from the Cartesian product of the domains of its associated letter, padding, and carry variables.

## Constraints

- All letter variables must represent distinct digits. For each pair $ (X_i, X_j) $, $ X_i \neq X_j $.
- The first letter of each input word (a leading variable $ X_k $) cannot be zero: $ X_k \neq 0 $.
- For each hidden variable $ U_k $, its associated letter or padding variables $ A_1, A_2, A_3 \in \{P\} \cup \{ X_1, X_2, \dots, X_n \} $ and carry variables $ B_1, B_2 \in \{ C_1, C_2, \dots, C_d \} $ must match the original variables and must satisfy the constraint: $ B_1 + A_1 + A_2 = 10 B_2 + A_3 $.

## Arc Consistency

To solve the cryptarithmetic problem efficiently, the solver employs arc consistency (specifically, the AC-3 algorithm) to enforce consistency across the constraints in the CSP. Arc consistency ensures that for every value in the domain of a variable, there exists a compatible value in the domain of each related variable, reducing the search space before backtracking

---