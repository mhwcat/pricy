# pricy
Simple CLI tool for tracking prices in online stores and sending e-mail notifications when price changes.
## Building
[Install Rustlang](https://www.rust-lang.org/tools/install).
Build release version:
```bash
cargo build --release
```
Build release version without email support:
```bash
cargo build --release --no-default-features
```
## Configuration
Pricy requires configuration toml file where URLs, HTML selectors and optionally SMTP credentials are provided.  
You have to provide HTML selector where price is stored on the website in CSS-like syntax. 
```html
<div class="price">1899.00</div>
```
```toml
selector = "div.price"
```
By default inner text value from given selector is used, but you can optionally specify `use_selector_attr` property which indicates that provided attribute of selector should be used instead, for example:
```html
<span id="price-field" content="123.10">Price is one-two-three-point-ten</span>
```
```toml
selector = "span#price-field"
use_selector_attr = "content"
```
You can also specify
```toml
notify_only_drop = true
```
for each product in order to be notified only when price drops.  
If you want to override default email notification recipients for a product, set 
```toml
notification_email_recipients = ["SomeRecipient3 <some@recipient3.dom>"]
```

Example configuration file is [here](example.toml).
## Running
You have to provide configuration file path and database file path when running pricy.
```bash
pricy --database db.ron --config config.toml
```
You can display help information with `--help`.
### Example output 
```
➜  pricy ./pricy -d db.ron -c config.toml
Fetching https://some-store.domain/product-1.html
Updating price for product https://some-store.domain/product-1.html: 1517.09 -> 1500.00 (last check at 2022-11-15 11:05:38 UTC)
Sent email notification to Some Name <some@email>
```
## E-mail support
Pricy uses [lettre](https://github.com/lettre/lettre) for sending e-mails. It assumes secure connection with SMTP server. `sender` and `recipients` configuration properties have to be provided in `SomeName <some@email>` format in order to be parsed correctly.