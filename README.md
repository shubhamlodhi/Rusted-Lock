<div align="center">

# 🔐 Rusted-Lock

<img src="https://drive.google.com/uc?id=1UPxdfYVgDf1uFJKaB5oVVt5IUN9QQtDX" width="120">

### **Experience a blazing-fast, secure authentication engine built in Rust**


[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Postgres](https://img.shields.io/badge/postgres-%23316192.svg?style=for-the-badge&logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Axum](https://img.shields.io/badge/axum-FFD43B?style=for-the-badge&logo=rust&logoColor=black)](https://github.com/tokio-rs/axum)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![API Docs](https://img.shields.io/badge/docs-API-blue)](docs/API.md)
[![GitHub Stars](https://img.shields.io/github/stars/shubhamlodhi/Rusted-Lock?style=social)](https://github.com/yourusername/Rusted-Lock/stargazers)

<img src="https://media2.giphy.com/media/v1.Y2lkPTc5MGI3NjExejZtdzFyYXg1ZGF5Ymg2b2MzYWtuZjh2MWduajhvZHJ3eDRzYjhoMyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9dg/xccxybHUpzoArffQQT/giphy.gif" width="400">
</div>

---

## 📖 Overview

**Rusted-Lock** delivers a state-of-the-art authentication solution built entirely in Rust. By harnessing the power of Rust's speed and safety, Axum's asynchronous web framework, and PostgreSQL via Diesel ORM, it offers a production-ready REST API designed for secure, scalable operations.

---

## ✨ Key Features

- 🏃‍♂️ **Optimized Speed:** Engineered with Rust and Axum to achieve lightning-fast performance.
- 🛡️ **Enterprise-Grade Security:** Equipped with advanced security measures to safeguard your data.
- 🎯 **Robust Type Safety:** Leverages Rust’s strong type system to reduce runtime errors.
- 🔄 **Complete Authentication Suite:** Provides a full range of authentication APIs.
- 📦 **Efficient Connection Pooling:** Manages database connections seamlessly.
- 🔍 **Robust Error Handling:** Features detailed error management for smooth operation.

<hr></hr>

## 🚀 Core Features

- ⚡ **Speed & Security:** Built in Rust for unmatched performance and reliability.
- 📈 **Scalability:** Architected to handle high traffic effortlessly.
- 🔧 **Extensibility:** Designed to easily incorporate new features and endpoints.
- 🗄️ **Seamless Database Integration:** Effortlessly connects with PostgreSQL via Diesel ORM.

<hr></hr>

## 🛠️ Usage

### API Endpoints

#### 🔐 Authentication
- **POST** `/api/login` – Authenticate user login.
- **POST** `/api/register` – Register a new account.
- **POST** `/api/logout` – Terminate the user session.

#### 👤 Users
- **GET** `/api/users` – Retrieve a list of all users.
- **POST** `/api/users` – Create a new user entry.
- **GET** `/api/users/{id}` – Fetch details for a specific user.
- **PUT** `/api/users/{id}` – Update information for a user.
- **DELETE** `/api/users/{id}` – Remove a user from the system.

<hr></hr>

## 🏃 How to Run

### Prerequisites

- **Rust**: Ensure you have Rust installed. You can install it from [rust-lang.org](https://www.rust-lang.org/).
- **PostgreSQL**: Make sure PostgreSQL is installed and running. You can download it from [postgresql.org](https://www.postgresql.org/).

### Setup

1. **Clone the repository**:
    ```sh
    git clone https://github.com/yourusername/Rusted-Lock.git
    cd Rusted-Lock
    ```

2. **Set up environment variables**:
    Create a `.env` file in the root directory and add the following:
    ```dotenv
    DATABASE_URL=postgres://postgres:root@localhost:5432/database_name
    PORT=3000
    HOST=127.0.0.1
    RUST_LOG=info
    MAX_DB_CONNECTIONS=5
    ```

3. **Install dependencies**:
    ```sh
    cargo build
    ```

4. **Run database migrations**:
    ```sh
   diesel setup
    diesel migration run
    ```

### Running the Application

1. **Start the server**:
    ```sh
    cargo run
    ```

2. **Access the API**:
    Open your browser or API client and navigate to `http://127.0.0.1:3000/api`.

<hr></hr>

## 📄 License

This project is distributed under the **MIT License**. For full details, refer to the [LICENSE](LICENSE) file.

<hr></hr>

<div align="center">
  <img src="https://media.giphy.com/media/78XCFBGOlS6keY1Bil/giphy.gif?cid=790b76118yy5y0ek5vx1iacaboo2fy811rzwl0vf507hlbl9&ep=v1_gifs_search&rid=giphy.gif&ct=g" alt="Star" width="300">
  <p><strong>If you like this project 💖, please give it a star ⭐ !</strong></p>
</div>