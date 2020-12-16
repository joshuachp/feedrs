use rss::{Channel, ChannelBuilder, Item, ItemBuilder};
use sqlx::{sqlite::SqliteConnectOptions, Executor, FromRow, SqlitePool};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};
use syndication::Feed;
use tokio::sync::mpsc;

#[derive(FromRow)]
struct SqlRssChannel {
    source: String,
    title: String,
    link: String,
    description: String,
    language: Option<String>,
    copyright: Option<String>,
    managing_editor: Option<String>,
    webmaster: Option<String>,
    pub_date: Option<String>,
    last_build_date: Option<String>,
    generator: Option<String>,
    docs: Option<String>,
    rating: Option<String>,
    ttl: Option<String>,
}

#[derive(FromRow)]
struct SqlRssItem {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    author: Option<String>,
    comments: Option<String>,
    pub_date: Option<String>,
    content: Option<String>,
}

impl From<SqlRssChannel> for Channel {
    fn from(channel: SqlRssChannel) -> Self {
        ChannelBuilder::default()
            .title(channel.title)
            .link(channel.link)
            .description(channel.description)
            .language(channel.language)
            .copyright(channel.copyright)
            .managing_editor(channel.managing_editor)
            .webmaster(channel.webmaster)
            .pub_date(channel.pub_date)
            .last_build_date(channel.last_build_date)
            .generator(channel.generator)
            .docs(channel.docs)
            .rating(channel.rating)
            .ttl(channel.ttl)
            .build()
            .unwrap()
    }
}

impl From<SqlRssItem> for Item {
    fn from(item: SqlRssItem) -> Self {
        ItemBuilder::default()
            .title(item.title)
            .link(item.link)
            .description(item.description)
            .author(item.author)
            .comments(item.comments)
            .pub_date(item.pub_date)
            .content(item.content)
            .build()
            .unwrap()
    }
}

pub async fn create_database(path: &Path) -> sqlx::Result<Arc<SqlitePool>> {
    // The pool create asynchronously
    let pool = Arc::new(SqlitePool::connect_lazy_with(
        SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true),
    ));

    // TODO: Log error
    let mut trans = pool.begin().await?;

    // TODO: Database tables
    trans
        .execute(
            "CREATE TABLE IF NOT EXISTS Sources (
                sources TEXT PRIMARY KEY
            )",
        )
        .await?;
    trans
        .execute(
            // TODO: categories: Vec<Category>, cloud: Option<Cloud>
            // A text input box that can be displayed with the channel.
            // text_input: Option<TextInput>,
            // A hint to tell the aggregator which hours it can skip.
            // skip_hours: Vec<String>,
            // A hint to tell the aggregator which days it can skip.
            // skip_days: Vec<String>,
            // The items in the channel.
            // items: Vec<Item>,
            // The extensions for the channel.
            // extensions: ExtensionMap,
            // The iTunes extension for the channel.
            // itunes_ext: Option<itunes::ITunesChannelExtension>,
            // The Dublin Core extension for the channel.
            // dublin_core_ext: Option<dublincore::DublinCoreExtension>,
            // The Syndication extension for the channel.
            // syndication_ext: Option<syndication::SyndicationExtension>,
            // The namespaces present in the RSS tag.
            // namespaces: HashMap<String, String>,
            // image
            "CREATE TABLE IF NOT EXISTS RSS_Channels (
                source TEXT
                    PRIMARY KEY 
                    REFERENCES Sources (source)
                    ON DELETE CASCADE
                    ON UPDATE CASCADE,
                title TEXT NOT NULL,
                link TEXT NOT NULL,
                description TEXT NOT NULL,
                language TEXT,
                copyright TEXT,
                managing_editor TEXT,
                webmaster TEXT,
                pub_date TEXT,
                last_build_date TEXT,
                generator TEXT,
                docs TEXT,
                rating TEXT,
                ttl TEXT
            )",
        )
        .await?;
    trans
        .execute(
            // TODO: Following
            // The categories the item belongs to.
            // categories: Vec<Category>,
            // The description of a media object that is attached to the item.
            // enclosure: Option<Enclosure>,
            // NOTE: Maybe primary key
            // A unique identifier for the item.
            // guid: Option<Guid>,
            // The extensions for the item.
            // extensions: ExtensionMap,
            // The iTunes extension for the item.
            // itunes_ext: Option<itunes::ITunesItemExtension>,
            // The Dublin Core extension for the item.
            // dublin_core_ext: Option<dublincore::DublinCoreExtension>,
            "CREATE TABLE IF NOT EXISTS RSS_Items (
                source TEXT NOT NULL
                    REFERENCES RSS_Channels (source)
                    ON DELETE CASCADE
                    ON UPDATE CASCADE,
                title TEXT,
                link TEXT,
                description TEXT,
                author TEXT,
                comments TEXT,
                pub_date TEXT,
                content TEXT
            )",
        )
        .await?;
    // trans.execute("CREATE TABLE IF NOT EXISTS Atom_Feeds ()");
    // TODO: Check result
    trans.commit().await?;

    Ok(pool)
}

pub async fn get_all(
    pool: &Arc<SqlitePool>,
    content: &RwLock<HashMap<Arc<String>, Feed>>,
) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    let channels: Vec<SqlRssChannel> = sqlx::query_as(
        "SELECT 
            source
            title,
            link,
            description,
            language,
            copyright,
            managing_editor,
            webmaster,
            pub_date,
            last_build_date,
            generator,
            docs,
            rating,
            ttl
        FROM RSS_Channels",
    )
    .fetch_all(&mut conn)
    .await?;

    if channels.len() > 0 {
        let (tx, mut rx) = mpsc::channel(channels.len());
        for channel in channels {
            let pool = Arc::clone(pool);
            let mut tx = tx.clone();
            tokio::spawn(async move {
                let source = Arc::new(channel.source.clone());
                let items = get_rss_items(&source, &pool).await.unwrap();
                let mut channel = Channel::from(channel);
                channel.set_items(items);
                tx.send((source, channel)).await.unwrap();
            });
        }
        drop(tx);

        let mut content = content.write().unwrap();
        while let Some((source, res)) = rx.recv().await {
            content.insert(source, Feed::RSS(res));
        }
    }
    Ok(())
}

async fn get_rss_items(source: &str, pool: &Arc<SqlitePool>) -> sqlx::Result<Vec<Item>> {
    let mut conn = pool.acquire().await?;
    let items: Vec<SqlRssItem> = sqlx::query_as(
        "SELECT
            title,
            link,
            description,
            author,
            comments,
            pub_date,
            content
        FROM RSS_Items
        WHERE source = ?",
    )
    .bind(source)
    .fetch_all(&mut conn)
    .await?;
    let items: Vec<Item> = items.into_iter().map(|x| Item::from(x)).collect();
    Ok(items)
}
