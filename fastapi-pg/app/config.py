from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    DATABASE_PORT: int
    POSTGRES_PASSWORD: str
    POSTGRES_USER: str
    POSTGRES_DB: str
    POSTGRES_HOST: str

    class Config:
        env_file = '../.env'
        env_file_encoding = 'utf-8'
