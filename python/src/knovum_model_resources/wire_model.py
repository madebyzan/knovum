from pydantic import BaseModel, ConfigDict


class WireModel(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)
