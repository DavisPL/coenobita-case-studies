#[test]
fn test_fcntl_lock() {
    use rustix::fs::{fcntl_lock, FlockOperation};

    let f = tempfile::tempfile(|p1, p2| std::path::Path::new(p1).join(p2)).unwrap();
    fcntl_lock(&f, FlockOperation::LockExclusive).unwrap();
    fcntl_lock(&f, FlockOperation::Unlock).unwrap();
    let g = tempfile::tempfile(|p1, p2| std::path::Path::new(p1).join(p2)).unwrap();
    fcntl_lock(&g, FlockOperation::LockExclusive).unwrap();
    fcntl_lock(&g, FlockOperation::Unlock).unwrap();
    drop(f);
    drop(g);

    let f = tempfile::tempfile(|p1, p2| std::path::Path::new(p1).join(p2)).unwrap();
    fcntl_lock(&f, FlockOperation::LockShared).unwrap();
    let g = tempfile::tempfile(|p1, p2| std::path::Path::new(p1).join(p2)).unwrap();
    fcntl_lock(&g, FlockOperation::LockShared).unwrap();
    fcntl_lock(&f, FlockOperation::Unlock).unwrap();
    fcntl_lock(&g, FlockOperation::Unlock).unwrap();
    drop(f);
    drop(g);

    let f = tempfile::tempfile(|p1, p2| std::path::Path::new(p1).join(p2)).unwrap();
    fcntl_lock(&f, FlockOperation::LockShared).unwrap();
    fcntl_lock(&f, FlockOperation::LockExclusive).unwrap();
    fcntl_lock(&f, FlockOperation::Unlock).unwrap();
    let g = tempfile::tempfile(|p1, p2| std::path::Path::new(p1).join(p2)).unwrap();
    fcntl_lock(&g, FlockOperation::LockShared).unwrap();
    fcntl_lock(&g, FlockOperation::LockExclusive).unwrap();
    fcntl_lock(&g, FlockOperation::Unlock).unwrap();
    drop(f);
    drop(g);
}
