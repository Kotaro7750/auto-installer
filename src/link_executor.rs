use crate::argument_resolver::ArgumentResolver;
use crate::schema::PathStr;
use std::path::{Path, PathBuf};

pub trait LinkExecutor: ArgumentResolver {
    fn create_link(&self, original: &Path, link: &Path) -> Result<(), Box<dyn std::error::Error>>;

    fn execute_link(
        &self,
        original: &PathStr,
        link: &PathStr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let original_path = PathBuf::from(self.resolve_path_argument(original));
        // 実際にはファイルの存在だけではなくmetadataの取得に必要なパーミッションがないときにもエラーを出す
        // これがなかったらどうせ現在のユーザーが読み取れないのでエラーにしてもよいはず
        // cf. https://doc.rust-lang.org/std/fs/fn.metadata.html#errors
        std::fs::metadata(&original_path)?;

        let link_path = PathBuf::from(self.resolve_path_argument(link));
        // 今から張るリンクは存在してはならないが存在しているとリンクを張る段階でエラーが出るはず

        println!(
            "create symlink original: `{}` link: `{}`",
            original_path.display(),
            link_path.display()
        );

        self.create_link(&original_path, &link_path)
    }
}
