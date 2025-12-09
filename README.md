# Solana Optimum Presale Ladder

A smart contract developed the optimum single ladder is $0.01 → $0.05 across 5 stages, raising ~$31.75M presale and achieving a $50M market cap ceiling. This balances fast accumulation with attractive entry pricing. 
Built using the **Anchor framework** on Solana.

---

## Prerequisites

Before you begin, make sure you have the following tools installed:

- **[Rust](https://rust-lang.org/tools/install/)** – The Rust programming language and toolchain.  
- **[Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)** – Rust’s package manager.  
- **[Anchor CLI](https://project-serum.github.io/anchor/getting-started/installation.html)** – Command-line interface for Solana Anchor development.  
- **[Node.js](https://nodejs.org/en/download/)** – JavaScript runtime for running scripts.  
- **[Yarn](https://yarnpkg.com/getting-started/install)** – Package manager for JavaScript dependencies.  

---

## Getting Started

1. **Installation:** Clone the repository and install dependencies.

   ```bash
   git clone https://github.com/shivamseasia/growsol.git
   cd growsol
   yarn
   ```

2. **Build the Smart Contract:**

   ```bash
   anchor build
   ```

3. **Run Tests:**

   ```bash
   anchor test
   ```

4. **Deploy:**

   Switch to your desired network and deploy
   ```bash
   anchor deploy
   ```