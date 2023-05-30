Git = {}
Git.__index = Git

-- Create a new Git instance
function Git:new(repositoryPath)
    local self = setmetatable({}, Git)
    self.repositoryPath = repositoryPath
    --self.branches = self:_git_get_branches()
    self.rootPath = _get_git_project_root_path(repositoryPath)
    self.branches = _get_git_project_branches(self.rootPath)
    return self
end