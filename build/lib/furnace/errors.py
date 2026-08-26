class ForgeError(Exception):
    """A user-facing compiler error. Carries source position when known."""
    def __init__(self, message, line=None, col=None):
        self.line = line
        self.col = col
        super().__init__(message)

    def render(self, filename=None):
        loc = f"{filename}:" if filename else ""
        if self.line:
            loc += f"{self.line}:{self.col or 0}: "
        return f"error: {loc}{self}"