import yummy
def pre_deviceid_auth(model):
    if model.get_device_id() == "erhanbaris":
        yummy.fail("erhanbaris kullanilamaz")
    print(model)


