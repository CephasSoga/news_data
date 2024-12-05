# **Financial News Collector**

A Rust-based application that fetches, stores, and provides financial news from APIs like Alpha Vantage and MarketAux. The app uses MongoDB for efficient data storage and supports real-time updates for staying current with financial trends.

---

## **Features**
- 📰 Fetches financial news from APIs (Alpha Vantage, MarketAux, etc.).
- 📊 Categorizes news by symbols, topics, and sentiment.
- 💾 Stores news in MongoDB for querying and analysis.
- 🔄 Real-time updates with configurable intervals.
- 🌐 API endpoints for serving collected news (optional).

---

## **Tech Stack**
- **Backend**: Rust with async capabilities (Reqwest, Tokio).
- **Database**: MongoDB for flexible and scalable storage.
- **APIs**: 
  - Alpha Vantage for sentiment-rich news.
  - MarketAux for categorized news and topics.
- **Hosting**: Cloud-based deployment (AWS, DigitalOcean, or Railway).

---

## **Project Structure**
```
financial_news_collector/
│
├── src/
│   ├── main.rs         # Main entry point of the app
│   ├── db.rs           # MongoDB connection and queries
│   ├── api.rs          # Logic for API integration
│   ├── models.rs       # Data models for MongoDB and APIs
│   ├── scheduler.rs    # Task scheduler for periodic updates
│   └── web.rs          # (Optional) API endpoints with Axum/Actix Web
│
├── .env               # Environment variables (API keys, MongoDB URI)
├── Cargo.toml         # Rust dependencies and project configuration
├── README.md          # Project overview and instructions
└── LICENSE            # License file
```

---

## **Setup and Installation**

### **Prerequisites**
- Install **Rust**: [Install guide](https://www.rust-lang.org/tools/install)
- Install **MongoDB**: [Download MongoDB](https://www.mongodb.com/try/download/community)
- Get API keys for:
  - [Alpha Vantage](https://www.alphavantage.co/support/#api-key)
  - [MarketAux](https://marketaux.com/)

---

### **Steps**
1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-username/financial_news_collector.git
   cd financial_news_collector
   ```

2. **Set up environment variables**:
   Create a `.env` file:
   ```
   MONGO_URI=mongodb://localhost:27017
   ALPHA_VANTAGE_KEY=your_alpha_vantage_api_key
   MARKETAUX_KEY=your_marketaux_api_key
   ```

3. **Run the application**:
   ```bash
   cargo run
   ```

---

## **Roadmap**

### Phase 1: Core Features
- [x] Set up MongoDB connection.
- [x] Fetch news data from MarketAux and Alpha Vantage.
- [x] Save news data in MongoDB.

### Phase 2: Real-Time Updates
- [x] Implement a scheduler for periodic API calls.
- [ ] Add error handling and retries for API requests.

### Phase 3: Serve News Data
- [ ] Create API endpoints with Axum/Actix Web.
- [ ] Add filtering by ticker, category, or sentiment.

### Phase 4: Optimization and Deployment
- [ ] Optimize MongoDB queries with indices.
- [ ] Deploy the application to a cloud platform.

---

## **Usage**

### Fetching News
The app periodically fetches news articles from the configured APIs and saves them to MongoDB.

### Example News Document in MongoDB
```json
{
  "title": "Apple Inc. Reports Record Earnings",
  "link": "https://example.com/apple-earnings",
  "summary": "Apple Inc. announced record earnings for Q4 2023...",
  "date": "2023-12-05",
  "sentiment": "positive"
}
```

### Accessing News (Future Feature)
- Endpoint: `/news`
- Query Parameters: `symbol`, `date_from`, `date_to`, `sentiment`.

---

## **Contributing**
Feel free to open issues or submit pull requests to improve the app!

---

## **License**
This project is licensed under the MIT License. See the `LICENSE` file for details.

---

## **Contact**
For questions or collaboration:
- **Author**: Cephas N. Soga
- **Email**: cephassoga@gmail.com