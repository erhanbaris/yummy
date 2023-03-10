def pre_deviceid_auth(model):
    if model.id == "erhanbaris":
        raise Exception("erhanbaris kullanilamaz")
    print(model)

def post_deviceid_auth(model, successed):
    print(model.__dict__)
    print(successed)

