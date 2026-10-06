from pydantic import BaseModel, ConfigDict


class WireModel(BaseModel):
    model_config = ConfigDict(strict=True, populate_by_name=True)
