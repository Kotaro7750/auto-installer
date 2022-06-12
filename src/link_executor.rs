use std::path::PathBuf;

pub trait LinkExecutor {
    fn create_link(
        &self,
        original: &PathBuf,
        link: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn execute_link(
        &self,
        original: &String,
        link: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let original_path = PathBuf::from(original);
        // 実際にはファイルの存在だけではなくmetadataの取得に必要なパーミッションがないときにもエラーを出す
        // これがなかったらどうせ現在のユーザーが読み取れないのでエラーにしてもよいはず
        // cf. https://doc.rust-lang.org/std/fs/fn.metadata.html#errors
        std::fs::metadata(&original_path)?;

        let link_path = PathBuf::from(link);
        // 今から張るリンクは存在してはならないが存在しているとリンクを張る段階でエラーが出るはず

        println!("create symlink original: `{}` link: `{}`", original, link);

        self.create_link(&original_path, &link_path)
    }
}
