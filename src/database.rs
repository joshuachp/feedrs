use rss::{Channel, ChannelBuilder};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::Executor;
use sqlx::FromRow;
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::Arc;

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

pub fn create_database(path: &Path) -> sqlx::Result<Arc<SqlitePool>> {
    // The pool create asynchronously
    let pool = Arc::new(SqlitePool::connect_lazy_with(
        SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true),
    ));

    let t_pool = Arc::clone(&pool);
    tokio::spawn(async move {
        // TODO: Log error
        let mut trans = t_pool.begin().await.unwrap();

        // TODO: Database tables
        trans.execute(
            "CREATE TABLE IF NOT EXISTS Sources (
                sources TEXT PRIMARY KEY
            )",
        );
        trans.execute(
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
                ttl TEXT,
            )",
        );
        trans.execute(
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
                title TEXT,
                link TEXT,
                description TEXT,
                author TEXT,
                comments TEXT,
                pub_date TEXT,
                /// The RSS channel the item came from.
                source TEXT NOT NULL
                    REFERENCES RSS_Channels (source)
                    ON DELETE CASCADE
                    ON UPDATE CASCADE,
                content TEXT,
            )",
        );
        // trans.execute("CREATE TABLE IF NOT EXISTS Atom_Feeds ()");
        // TODO: Check result
        trans.commit().await.unwrap();
    });

    Ok(pool)
}

pub async fn get_all(pool: &Arc<SqlitePool>) -> sqlx::Result<()> {
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
            ttl,
            image,
        FROM RSS_Channels",
    )
    .fetch_all(&mut conn)
    .await?;
    let channels: Vec<Channel> = channels.into_iter().map(|x| Channel::from(x)).collect();

    Ok(())
}
